// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#include "FastStartupCore.h"
#include "Misc/Paths.h"
#include "HAL/FileManager.h"

DEFINE_LOG_CATEGORY(LogFastStartup);

#define LOCTEXT_NAMESPACE "FFastStartupCoreModule"

void FFastStartupCoreModule::StartupModule()
{
	UE_LOG(LogFastStartup, Log, TEXT("Fast Startup Core Module loaded"));
	
	// Initialize cache path
	CachePath = FPaths::ProjectDir() / TEXT("Saved") / TEXT("FastStartup") / TEXT("startup.uefast");
	
	// Check if we should enable fast startup
	if (IsCacheValid())
	{
		UE_LOG(LogFastStartup, Log, TEXT("Valid startup cache found: %s"), *CachePath);
		bEnabled = true;
	}
	else
	{
		UE_LOG(LogFastStartup, Log, TEXT("No valid startup cache found"));
		bEnabled = false;
	}
}

void FFastStartupCoreModule::ShutdownModule()
{
	UE_LOG(LogFastStartup, Log, TEXT("Fast Startup Core Module unloaded"));
}

FFastStartupCoreModule& FFastStartupCoreModule::Get()
{
	return FModuleManager::LoadModuleChecked<FFastStartupCoreModule>("FastStartupCore");
}

void FFastStartupCoreModule::SetEnabled(bool bInEnabled)
{
	bEnabled = bInEnabled;
	UE_LOG(LogFastStartup, Log, TEXT("Fast Startup %s"), bEnabled ? TEXT("enabled") : TEXT("disabled"));
}

FString FFastStartupCoreModule::GetCLIPath() const
{
	// Look for the CLI in the plugin's Binaries folder
	FString PluginDir = FPaths::ProjectPluginsDir() / TEXT("FastStartup");
	
#if PLATFORM_WINDOWS
	FString CLIName = TEXT("ue5-fast-startup.exe");
#else
	FString CLIName = TEXT("ue5-fast-startup");
#endif

	FString CLIPath = PluginDir / TEXT("Binaries") / CLIName;
	
	if (FPaths::FileExists(CLIPath))
	{
		return CLIPath;
	}
	
	// Fallback to project root
	CLIPath = FPaths::ProjectDir() / TEXT("Binaries") / CLIName;
	if (FPaths::FileExists(CLIPath))
	{
		return CLIPath;
	}
	
	return FString();
}

FString FFastStartupCoreModule::GetCachePath() const
{
	return CachePath;
}

bool FFastStartupCoreModule::IsCacheValid() const
{
	if (!FPaths::FileExists(CachePath))
	{
		return false;
	}
	
	// Check magic bytes
	TArray<uint8> FileData;
	if (!FFileHelper::LoadFileToArray(FileData, *CachePath))
	{
		return false;
	}
	
	if (FileData.Num() < 8)
	{
		return false;
	}
	
	// Check for "UEFAST01" magic
	const uint8 Magic[] = { 'U', 'E', 'F', 'A', 'S', 'T', '0', '1' };
	for (int32 i = 0; i < 8; ++i)
	{
		if (FileData[i] != Magic[i])
		{
			return false;
		}
	}
	
	return true;
}

#undef LOCTEXT_NAMESPACE
	
IMPLEMENT_MODULE(FFastStartupCoreModule, FastStartupCore)
