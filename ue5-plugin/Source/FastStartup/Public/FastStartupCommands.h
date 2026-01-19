// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#pragma once

#include "CoreMinimal.h"
#include "Framework/Commands/Commands.h"
#include "FastStartupStyle.h"

class FFastStartupCommands : public TCommands<FFastStartupCommands>
{
public:
	FFastStartupCommands()
		: TCommands<FFastStartupCommands>(
			TEXT("FastStartup"),
			NSLOCTEXT("Contexts", "FastStartup", "Fast Startup Accelerator"),
			NAME_None,
			FFastStartupStyle::GetStyleSetName())
	{
	}

	// TCommands<> interface
	virtual void RegisterCommands() override;

public:
	TSharedPtr<FUICommandInfo> OpenWindow;
	TSharedPtr<FUICommandInfo> AnalyzeProject;
	TSharedPtr<FUICommandInfo> BuildCache;
	TSharedPtr<FUICommandInfo> ToggleFastStartup;
};
