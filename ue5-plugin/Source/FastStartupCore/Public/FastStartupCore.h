// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"

DECLARE_LOG_CATEGORY_EXTERN(LogFastStartup, Log, All);

class FFastStartupCoreModule : public IModuleInterface
{
public:
	/** IModuleInterface implementation */
	virtual void StartupModule() override;
	virtual void ShutdownModule() override;

	/** Get singleton instance */
	static FFastStartupCoreModule& Get();

	/** Check if Fast Startup is enabled */
	bool IsEnabled() const { return bEnabled; }

	/** Enable/Disable Fast Startup */
	void SetEnabled(bool bInEnabled);

	/** Get the path to the Rust CLI executable */
	FString GetCLIPath() const;

	/** Get the path to the cache file */
	FString GetCachePath() const;

	/** Check if cache exists and is valid */
	bool IsCacheValid() const;

private:
	bool bEnabled = false;
	FString CachePath;
};
