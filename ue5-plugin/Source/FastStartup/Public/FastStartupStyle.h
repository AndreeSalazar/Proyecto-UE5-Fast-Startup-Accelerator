// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#pragma once

#include "CoreMinimal.h"
#include "Styling/SlateStyle.h"

class FFastStartupStyle
{
public:
	static void Initialize();
	static void Shutdown();
	static void ReloadTextures();
	static const ISlateStyle& Get();
	static FName GetStyleSetName();

private:
	static TSharedRef<class FSlateStyleSet> Create();
	static TSharedPtr<class FSlateStyleSet> StyleInstance;
};
