// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#pragma once

#include "CoreMinimal.h"
#include "Widgets/SCompoundWidget.h"
#include "Widgets/DeclarativeSyntaxSupport.h"

class SFastStartupWidget : public SCompoundWidget
{
public:
	SLATE_BEGIN_ARGS(SFastStartupWidget) {}
	SLATE_END_ARGS()

	void Construct(const FArguments& InArgs);

private:
	// UI Callbacks
	FReply OnAnalyzeClicked();
	FReply OnBuildCacheClicked();
	FReply OnVerifyCacheClicked();
	void OnFastStartupToggled(ECheckBoxState NewState);
	ECheckBoxState IsFastStartupEnabled() const;

	// Status
	FText GetStatusText() const;
	FSlateColor GetStatusColor() const;

	// Analysis results
	FText GetAssetCountText() const;
	FText GetCacheSizeText() const;
	FText GetEstimatedSavingsText() const;

	// State
	bool bIsAnalyzing = false;
	bool bIsBuildingCache = false;
	int32 TotalAssets = 0;
	int32 StartupAssets = 0;
	float EstimatedSavings = 0.0f;
	FString CacheStatus;
};
