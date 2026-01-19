// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#include "FastStartupStyle.h"
#include "Styling/SlateStyleRegistry.h"
#include "Framework/Application/SlateApplication.h"
#include "Slate/SlateGameResources.h"
#include "Interfaces/IPluginManager.h"
#include "Styling/SlateStyleMacros.h"

#define RootToContentDir Style->RootToContentDir

TSharedPtr<FSlateStyleSet> FFastStartupStyle::StyleInstance = nullptr;

void FFastStartupStyle::Initialize()
{
	if (!StyleInstance.IsValid())
	{
		StyleInstance = Create();
		FSlateStyleRegistry::RegisterSlateStyle(*StyleInstance);
	}
}

void FFastStartupStyle::Shutdown()
{
	FSlateStyleRegistry::UnRegisterSlateStyle(*StyleInstance);
	ensure(StyleInstance.IsUnique());
	StyleInstance.Reset();
}

FName FFastStartupStyle::GetStyleSetName()
{
	static FName StyleSetName(TEXT("FastStartupStyle"));
	return StyleSetName;
}

const ISlateStyle& FFastStartupStyle::Get()
{
	return *StyleInstance;
}

void FFastStartupStyle::ReloadTextures()
{
	if (FSlateApplication::IsInitialized())
	{
		FSlateApplication::Get().GetRenderer()->ReloadTextureResources();
	}
}

TSharedRef<FSlateStyleSet> FFastStartupStyle::Create()
{
	TSharedRef<FSlateStyleSet> Style = MakeShareable(new FSlateStyleSet(GetStyleSetName()));
	Style->SetContentRoot(IPluginManager::Get().FindPlugin("FastStartup")->GetBaseDir() / TEXT("Resources"));

	// Define styles
	Style->Set("FastStartup.OpenWindow", new IMAGE_BRUSH_SVG(TEXT("Icon128"), CoreStyleConstants::Icon16x16));
	Style->Set("FastStartup.AnalyzeProject", new IMAGE_BRUSH_SVG(TEXT("Icon128"), CoreStyleConstants::Icon16x16));
	Style->Set("FastStartup.BuildCache", new IMAGE_BRUSH_SVG(TEXT("Icon128"), CoreStyleConstants::Icon16x16));

	return Style;
}

#undef RootToContentDir
