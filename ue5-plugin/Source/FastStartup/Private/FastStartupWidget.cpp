// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#include "FastStartupWidget.h"
#include "FastStartupCore.h"

#include "Widgets/Layout/SBox.h"
#include "Widgets/Layout/SBorder.h"
#include "Widgets/Layout/SScrollBox.h"
#include "Widgets/Layout/SSeparator.h"
#include "Widgets/Text/STextBlock.h"
#include "Widgets/Input/SButton.h"
#include "Widgets/Input/SCheckBox.h"
#include "Widgets/SBoxPanel.h"
#include "Styling/AppStyle.h"
#include "HAL/PlatformProcess.h"
#include "Misc/Paths.h"

#define LOCTEXT_NAMESPACE "SFastStartupWidget"

void SFastStartupWidget::Construct(const FArguments& InArgs)
{
	// Check initial cache status
	FFastStartupCoreModule& Core = FFastStartupCoreModule::Get();
	CacheStatus = Core.IsCacheValid() ? TEXT("Valid") : TEXT("Not Found");

	ChildSlot
	[
		SNew(SScrollBox)
		+ SScrollBox::Slot()
		.Padding(16.0f)
		[
			SNew(SVerticalBox)

			// Header
			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 0, 0, 16)
			[
				SNew(STextBlock)
				.Text(LOCTEXT("Title", "Fast Startup Accelerator"))
				.Font(FCoreStyle::GetDefaultFontStyle("Bold", 18))
			]

			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 0, 0, 8)
			[
				SNew(STextBlock)
				.Text(LOCTEXT("Subtitle", "Reduce Unreal Engine 5 editor startup times"))
				.Font(FCoreStyle::GetDefaultFontStyle("Regular", 10))
				.ColorAndOpacity(FSlateColor::UseSubduedForeground())
			]

			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 0, 0, 16)
			[
				SNew(SSeparator)
			]

			// Enable Toggle
			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 0, 0, 16)
			[
				SNew(SHorizontalBox)
				+ SHorizontalBox::Slot()
				.AutoWidth()
				.VAlign(VAlign_Center)
				[
					SNew(SCheckBox)
					.IsChecked(this, &SFastStartupWidget::IsFastStartupEnabled)
					.OnCheckStateChanged(this, &SFastStartupWidget::OnFastStartupToggled)
				]
				+ SHorizontalBox::Slot()
				.AutoWidth()
				.VAlign(VAlign_Center)
				.Padding(8, 0, 0, 0)
				[
					SNew(STextBlock)
					.Text(LOCTEXT("EnableFastStartup", "Enable Fast Startup Mode"))
					.Font(FCoreStyle::GetDefaultFontStyle("Bold", 12))
				]
			]

			// Status
			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 0, 0, 16)
			[
				SNew(SBorder)
				.BorderImage(FAppStyle::GetBrush("ToolPanel.GroupBorder"))
				.Padding(8)
				[
					SNew(SVerticalBox)
					+ SVerticalBox::Slot()
					.AutoHeight()
					[
						SNew(SHorizontalBox)
						+ SHorizontalBox::Slot()
						.AutoWidth()
						[
							SNew(STextBlock)
							.Text(LOCTEXT("Status", "Status: "))
							.Font(FCoreStyle::GetDefaultFontStyle("Bold", 10))
						]
						+ SHorizontalBox::Slot()
						.AutoWidth()
						[
							SNew(STextBlock)
							.Text(this, &SFastStartupWidget::GetStatusText)
							.ColorAndOpacity(this, &SFastStartupWidget::GetStatusColor)
						]
					]
					+ SVerticalBox::Slot()
					.AutoHeight()
					.Padding(0, 4, 0, 0)
					[
						SNew(STextBlock)
						.Text(this, &SFastStartupWidget::GetAssetCountText)
					]
					+ SVerticalBox::Slot()
					.AutoHeight()
					.Padding(0, 4, 0, 0)
					[
						SNew(STextBlock)
						.Text(this, &SFastStartupWidget::GetEstimatedSavingsText)
					]
				]
			]

			// Actions
			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 0, 0, 8)
			[
				SNew(STextBlock)
				.Text(LOCTEXT("Actions", "Actions"))
				.Font(FCoreStyle::GetDefaultFontStyle("Bold", 12))
			]

			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 0, 0, 8)
			[
				SNew(SHorizontalBox)
				+ SHorizontalBox::Slot()
				.AutoWidth()
				.Padding(0, 0, 8, 0)
				[
					SNew(SButton)
					.Text(LOCTEXT("Analyze", "Analyze Project"))
					.OnClicked(this, &SFastStartupWidget::OnAnalyzeClicked)
					.IsEnabled_Lambda([this]() { return !bIsAnalyzing; })
				]
				+ SHorizontalBox::Slot()
				.AutoWidth()
				.Padding(0, 0, 8, 0)
				[
					SNew(SButton)
					.Text(LOCTEXT("BuildCache", "Build Cache"))
					.OnClicked(this, &SFastStartupWidget::OnBuildCacheClicked)
					.IsEnabled_Lambda([this]() { return !bIsBuildingCache; })
				]
				+ SHorizontalBox::Slot()
				.AutoWidth()
				[
					SNew(SButton)
					.Text(LOCTEXT("VerifyCache", "Verify Cache"))
					.OnClicked(this, &SFastStartupWidget::OnVerifyCacheClicked)
				]
			]

			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 16, 0, 0)
			[
				SNew(SSeparator)
			]

			// Info
			+ SVerticalBox::Slot()
			.AutoHeight()
			.Padding(0, 16, 0, 0)
			[
				SNew(STextBlock)
				.Text(LOCTEXT("Info", "This plugin uses a Rust-powered CLI to analyze assets and build an optimized startup cache. The cache contains asset hashes, dependency graphs, and optimal load order to minimize editor startup time."))
				.AutoWrapText(true)
				.ColorAndOpacity(FSlateColor::UseSubduedForeground())
			]
		]
	];
}

FReply SFastStartupWidget::OnAnalyzeClicked()
{
	bIsAnalyzing = true;

	FFastStartupCoreModule& Core = FFastStartupCoreModule::Get();
	FString CLIPath = Core.GetCLIPath();

	if (!CLIPath.IsEmpty())
	{
		FString ProjectPath = FPaths::ProjectDir();
		FString OutputPath = FPaths::ProjectDir() / TEXT("Saved/FastStartup/analysis.json");
		FString Args = FString::Printf(TEXT("analyze --project \"%s\" --output \"%s\""), *ProjectPath, *OutputPath);

		FPlatformProcess::CreateProc(*CLIPath, *Args, true, false, false, nullptr, 0, nullptr, nullptr);
	}

	bIsAnalyzing = false;
	return FReply::Handled();
}

FReply SFastStartupWidget::OnBuildCacheClicked()
{
	bIsBuildingCache = true;

	FFastStartupCoreModule& Core = FFastStartupCoreModule::Get();
	FString CLIPath = Core.GetCLIPath();

	if (!CLIPath.IsEmpty())
	{
		FString ProjectPath = FPaths::ProjectDir();
		FString CachePath = Core.GetCachePath();
		FString Args = FString::Printf(TEXT("cache --project \"%s\" --output \"%s\" --force"), *ProjectPath, *CachePath);

		FPlatformProcess::CreateProc(*CLIPath, *Args, true, false, false, nullptr, 0, nullptr, nullptr);
		CacheStatus = TEXT("Building...");
	}

	bIsBuildingCache = false;
	return FReply::Handled();
}

FReply SFastStartupWidget::OnVerifyCacheClicked()
{
	FFastStartupCoreModule& Core = FFastStartupCoreModule::Get();
	
	if (Core.IsCacheValid())
	{
		CacheStatus = TEXT("Valid");
	}
	else
	{
		CacheStatus = TEXT("Invalid or Not Found");
	}

	return FReply::Handled();
}

void SFastStartupWidget::OnFastStartupToggled(ECheckBoxState NewState)
{
	FFastStartupCoreModule& Core = FFastStartupCoreModule::Get();
	Core.SetEnabled(NewState == ECheckBoxState::Checked);
}

ECheckBoxState SFastStartupWidget::IsFastStartupEnabled() const
{
	return FFastStartupCoreModule::Get().IsEnabled() ? ECheckBoxState::Checked : ECheckBoxState::Unchecked;
}

FText SFastStartupWidget::GetStatusText() const
{
	return FText::FromString(CacheStatus);
}

FSlateColor SFastStartupWidget::GetStatusColor() const
{
	if (CacheStatus == TEXT("Valid"))
	{
		return FSlateColor(FLinearColor::Green);
	}
	else if (CacheStatus == TEXT("Building..."))
	{
		return FSlateColor(FLinearColor::Yellow);
	}
	return FSlateColor(FLinearColor::Red);
}

FText SFastStartupWidget::GetAssetCountText() const
{
	if (TotalAssets > 0)
	{
		return FText::Format(LOCTEXT("AssetCount", "Assets: {0} total, {1} startup-critical"), TotalAssets, StartupAssets);
	}
	return LOCTEXT("AssetCountEmpty", "Assets: Run analysis to see counts");
}

FText SFastStartupWidget::GetCacheSizeText() const
{
	return LOCTEXT("CacheSize", "Cache: Check status above");
}

FText SFastStartupWidget::GetEstimatedSavingsText() const
{
	if (EstimatedSavings > 0)
	{
		return FText::Format(LOCTEXT("EstimatedSavings", "Estimated savings: {0}s"), FText::AsNumber(EstimatedSavings));
	}
	return LOCTEXT("EstimatedSavingsEmpty", "Estimated savings: Run analysis to calculate");
}

#undef LOCTEXT_NAMESPACE
