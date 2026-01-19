// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

#include "FastStartupCommands.h"

#define LOCTEXT_NAMESPACE "FFastStartupModule"

void FFastStartupCommands::RegisterCommands()
{
	UI_COMMAND(OpenWindow, "Fast Startup", "Open the Fast Startup Accelerator window", EUserInterfaceActionType::Button, FInputChord());
	UI_COMMAND(AnalyzeProject, "Analyze Project", "Analyze project assets and dependencies", EUserInterfaceActionType::Button, FInputChord());
	UI_COMMAND(BuildCache, "Build Cache", "Build startup cache for faster loading", EUserInterfaceActionType::Button, FInputChord());
	UI_COMMAND(ToggleFastStartup, "Enable Fast Startup", "Toggle fast startup mode", EUserInterfaceActionType::ToggleButton, FInputChord());
}

#undef LOCTEXT_NAMESPACE
