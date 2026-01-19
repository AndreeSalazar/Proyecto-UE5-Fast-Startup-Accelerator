// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"

class FToolBarBuilder;
class FMenuBuilder;

class FFastStartupModule : public IModuleInterface
{
public:
	/** IModuleInterface implementation */
	virtual void StartupModule() override;
	virtual void ShutdownModule() override;

	/** Get singleton instance */
	static FFastStartupModule& Get();

private:
	void RegisterMenus();
	TSharedRef<class SDockTab> OnSpawnPluginTab(const class FSpawnTabArgs& SpawnTabArgs);

	// Command handlers
	void OnOpenWindow();
	void OnAnalyzeProject();
	void OnBuildCache();
	void OnToggleFastStartup();
	bool IsFastStartupEnabled() const;

	TSharedPtr<class FUICommandList> PluginCommands;
};
