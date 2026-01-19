// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#include "FastStartupModule.h"
#include "FastStartupStyle.h"
#include "FastStartupCommands.h"
#include "FastStartupCore.h"
#include "FastStartupWidget.h"

#include "ToolMenus.h"
#include "Widgets/Docking/SDockTab.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Text/STextBlock.h"
#include "Framework/MultiBox/MultiBoxBuilder.h"
#include "LevelEditor.h"

static const FName FastStartupTabName("FastStartupTab");

#define LOCTEXT_NAMESPACE "FFastStartupModule"

void FFastStartupModule::StartupModule()
{
	FFastStartupStyle::Initialize();
	FFastStartupStyle::ReloadTextures();

	FFastStartupCommands::Register();

	PluginCommands = MakeShareable(new FUICommandList);

	PluginCommands->MapAction(
		FFastStartupCommands::Get().OpenWindow,
		FExecuteAction::CreateRaw(this, &FFastStartupModule::OnOpenWindow),
		FCanExecuteAction());

	PluginCommands->MapAction(
		FFastStartupCommands::Get().AnalyzeProject,
		FExecuteAction::CreateRaw(this, &FFastStartupModule::OnAnalyzeProject),
		FCanExecuteAction());

	PluginCommands->MapAction(
		FFastStartupCommands::Get().BuildCache,
		FExecuteAction::CreateRaw(this, &FFastStartupModule::OnBuildCache),
		FCanExecuteAction());

	PluginCommands->MapAction(
		FFastStartupCommands::Get().ToggleFastStartup,
		FExecuteAction::CreateRaw(this, &FFastStartupModule::OnToggleFastStartup),
		FCanExecuteAction(),
		FIsActionChecked::CreateRaw(this, &FFastStartupModule::IsFastStartupEnabled));

	UToolMenus::RegisterStartupCallback(FSimpleMulticastDelegate::FDelegate::CreateRaw(this, &FFastStartupModule::RegisterMenus));

	FGlobalTabmanager::Get()->RegisterNomadTabSpawner(FastStartupTabName, FOnSpawnTab::CreateRaw(this, &FFastStartupModule::OnSpawnPluginTab))
		.SetDisplayName(LOCTEXT("FastStartupTabTitle", "Fast Startup Accelerator"))
		.SetMenuType(ETabSpawnerMenuType::Hidden);
}

void FFastStartupModule::ShutdownModule()
{
	UToolMenus::UnRegisterStartupCallback(this);
	UToolMenus::UnregisterOwner(this);

	FFastStartupCommands::Unregister();
	FFastStartupStyle::Shutdown();

	FGlobalTabmanager::Get()->UnregisterNomadTabSpawner(FastStartupTabName);
}

FFastStartupModule& FFastStartupModule::Get()
{
	return FModuleManager::LoadModuleChecked<FFastStartupModule>("FastStartup");
}

TSharedRef<SDockTab> FFastStartupModule::OnSpawnPluginTab(const FSpawnTabArgs& SpawnTabArgs)
{
	return SNew(SDockTab)
		.TabRole(ETabRole::NomadTab)
		[
			SNew(SFastStartupWidget)
		];
}

void FFastStartupModule::RegisterMenus()
{
	FToolMenuOwnerScoped OwnerScoped(this);

	// Add to Window menu
	{
		UToolMenu* Menu = UToolMenus::Get()->ExtendMenu("LevelEditor.MainMenu.Window");
		FToolMenuSection& Section = Menu->FindOrAddSection("WindowLayout");
		Section.AddMenuEntryWithCommandList(FFastStartupCommands::Get().OpenWindow, PluginCommands);
	}

	// Add toolbar button
	{
		UToolMenu* ToolbarMenu = UToolMenus::Get()->ExtendMenu("LevelEditor.LevelEditorToolBar.PlayToolBar");
		FToolMenuSection& Section = ToolbarMenu->FindOrAddSection("PluginTools");
		
		FToolMenuEntry& Entry = Section.AddEntry(FToolMenuEntry::InitToolBarButton(FFastStartupCommands::Get().OpenWindow));
		Entry.SetCommandList(PluginCommands);
	}
}

void FFastStartupModule::OnOpenWindow()
{
	FGlobalTabmanager::Get()->TryInvokeTab(FastStartupTabName);
}

void FFastStartupModule::OnAnalyzeProject()
{
	FFastStartupCoreModule& Core = FFastStartupCoreModule::Get();
	FString CLIPath = Core.GetCLIPath();

	if (CLIPath.IsEmpty())
	{
		UE_LOG(LogFastStartup, Error, TEXT("CLI executable not found"));
		return;
	}

	FString ProjectPath = FPaths::ProjectDir();
	FString Args = FString::Printf(TEXT("analyze --project \"%s\" --output \"%s\""),
		*ProjectPath,
		*(FPaths::ProjectDir() / TEXT("Saved/FastStartup/analysis.json")));

	FProcHandle Handle = FPlatformProcess::CreateProc(*CLIPath, *Args, true, false, false, nullptr, 0, nullptr, nullptr);
	
	if (Handle.IsValid())
	{
		UE_LOG(LogFastStartup, Log, TEXT("Started project analysis"));
	}
}

void FFastStartupModule::OnBuildCache()
{
	FFastStartupCoreModule& Core = FFastStartupCoreModule::Get();
	FString CLIPath = Core.GetCLIPath();

	if (CLIPath.IsEmpty())
	{
		UE_LOG(LogFastStartup, Error, TEXT("CLI executable not found"));
		return;
	}

	FString ProjectPath = FPaths::ProjectDir();
	FString CachePath = Core.GetCachePath();
	FString Args = FString::Printf(TEXT("cache --project \"%s\" --output \"%s\" --force"),
		*ProjectPath,
		*CachePath);

	FProcHandle Handle = FPlatformProcess::CreateProc(*CLIPath, *Args, true, false, false, nullptr, 0, nullptr, nullptr);
	
	if (Handle.IsValid())
	{
		UE_LOG(LogFastStartup, Log, TEXT("Started cache build"));
	}
}

void FFastStartupModule::OnToggleFastStartup()
{
	FFastStartupCoreModule& Core = FFastStartupCoreModule::Get();
	Core.SetEnabled(!Core.IsEnabled());
}

bool FFastStartupModule::IsFastStartupEnabled() const
{
	return FFastStartupCoreModule::Get().IsEnabled();
}

#undef LOCTEXT_NAMESPACE

IMPLEMENT_MODULE(FFastStartupModule, FastStartup)
