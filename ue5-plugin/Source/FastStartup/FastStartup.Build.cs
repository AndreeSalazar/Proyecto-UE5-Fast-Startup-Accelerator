// Copyright 2026 Eddi Andreé Salazar Matos. Licensed under Apache 2.0.

using UnrealBuildTool;

public class FastStartup : ModuleRules
{
	public FastStartup(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = ModuleRules.PCHUsageMode.UseExplicitOrSharedPCHs;
		
		// UE 5.7.1 compatibility
		IWYUSupport = IWYUSupport.Full;
		
		PublicIncludePaths.AddRange(
			new string[] {
			}
		);
				
		PrivateIncludePaths.AddRange(
			new string[] {
			}
		);
			
		PublicDependencyModuleNames.AddRange(
			new string[]
			{
				"Core",
				"FastStartupCore"
			}
		);
			
		PrivateDependencyModuleNames.AddRange(
			new string[]
			{
				"CoreUObject",
				"Engine",
				"Slate",
				"SlateCore",
				"InputCore",
				"EditorFramework",
				"UnrealEd",
				"ToolMenus",
				"EditorStyle",
				"Projects",
				"Json",
				"JsonUtilities"
			}
		);
		
		DynamicallyLoadedModuleNames.AddRange(
			new string[]
			{
			}
		);
	}
}
