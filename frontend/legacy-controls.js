const LEGACY_CONTROLS = [
    {
        "form":  "ConfigForm.cs",
        "control":  "AccountCreate",
        "type":  "Button",
        "label":  "Create Character",
        "clickHandler":  "AccountCreate_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "ButtonViewCharacters",
        "type":  "Button",
        "label":  "View Characters",
        "clickHandler":  "ButtonViewCharacters_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "ClassOptionsButton",
        "type":  "Button",
        "label":  "Options",
        "clickHandler":  "ClassOptionsButton_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "CompileButton",
        "type":  "Button",
        "label":  "Test Compile",
        "clickHandler":  "CompileButton_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "DisplayHide",
        "type":  "RadioButton",
        "label":  "Hide game window",
        "clickHandler":  null
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "DisplayNormal",
        "type":  "RadioButton",
        "label":  "Leave game as normal",
        "clickHandler":  null
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "DisplayShrink",
        "type":  "RadioButton",
        "label":  "Shrink game window",
        "clickHandler":  null
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "EditDebuffs",
        "type":  "Button",
        "label":  "Manage",
        "clickHandler":  "EditDebuffs_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "EditKeymap",
        "type":  "Button",
        "label":  "Edit",
        "clickHandler":  "EditKeymap_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "LoadKeymap",
        "type":  "Button",
        "label":  "Reload from disk",
        "clickHandler":  "LoadKeymap_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "MyCancelButton",
        "type":  "Button",
        "label":  "Cancel",
        "clickHandler":  "MyCancelButton_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "MyHelpButton",
        "type":  "Button",
        "label":  "Help",
        "clickHandler":  "MyHelpButton_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "OKButton",
        "type":  "Button",
        "label":  "OK",
        "clickHandler":  "OKButton_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "PartyFollower",
        "type":  "RadioButton",
        "label":  "Follower",
        "clickHandler":  null
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "PartyLeader",
        "type":  "RadioButton",
        "label":  "Leader",
        "clickHandler":  null
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "PartySolo",
        "type":  "RadioButton",
        "label":  "Solo",
        "clickHandler":  null
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "SetInitial",
        "type":  "Button",
        "label":  "Set",
        "clickHandler":  "SetInitial_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "SetProfile1",
        "type":  "Button",
        "label":  "Set",
        "clickHandler":  "SetProfile1_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "SetProfile2",
        "type":  "Button",
        "label":  "Set",
        "clickHandler":  "SetProfile2_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "SetProfile3",
        "type":  "Button",
        "label":  "Set",
        "clickHandler":  "SetProfile3_Click"
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "VendGreen",
        "type":  "RadioButton",
        "label":  "Sell poor, common, and uncommon items",
        "clickHandler":  null
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "VendGrey",
        "type":  "RadioButton",
        "label":  "Sell only poor items",
        "clickHandler":  null
    },
    {
        "form":  "ConfigForm.cs",
        "control":  "VendWhite",
        "type":  "RadioButton",
        "label":  "Sell poor and common items",
        "clickHandler":  null
    },
    {
        "form":  "DebuffList.cs",
        "control":  "MyHelpButton",
        "type":  "Button",
        "label":  "Help",
        "clickHandler":  "MyHelpButton_Click"
    },
    {
        "form":  "DebuffList.cs",
        "control":  "MyOKButton",
        "type":  "Button",
        "label":  "Done",
        "clickHandler":  "MyOKButton_Click"
    },
    {
        "form":  "DebuffPick.cs",
        "control":  "ButtonCancel",
        "type":  "Button",
        "label":  "Cancel",
        "clickHandler":  "ButtonCancel_Click"
    },
    {
        "form":  "DebuffPick.cs",
        "control":  "ButtonCurse",
        "type":  "Button",
        "label":  "Curse",
        "clickHandler":  "ButtonCurse_Click"
    },
    {
        "form":  "DebuffPick.cs",
        "control":  "ButtonDisease",
        "type":  "Button",
        "label":  "Disease",
        "clickHandler":  "ButtonDisease_Click"
    },
    {
        "form":  "DebuffPick.cs",
        "control":  "ButtonMagic",
        "type":  "Button",
        "label":  "Magic",
        "clickHandler":  "ButtonMagic_Click"
    },
    {
        "form":  "DebuffPick.cs",
        "control":  "ButtonPoison",
        "type":  "Button",
        "label":  "Poison",
        "clickHandler":  "ButtonPoison_Click"
    },
    {
        "form":  "EvoConfigWindow.cs",
        "control":  "MyCancelButton",
        "type":  "Button",
        "label":  "Cancel",
        "clickHandler":  null
    },
    {
        "form":  "EvoConfigWindow.cs",
        "control":  "MyHelpButton",
        "type":  "Button",
        "label":  "Help",
        "clickHandler":  "MyHelpButton_Click"
    },
    {
        "form":  "EvoConfigWindow.cs",
        "control":  "MyOKButton",
        "type":  "Button",
        "label":  "OK",
        "clickHandler":  null
    },
    {
        "form":  "FactionReminder.cs",
        "control":  "MyHelpButton",
        "type":  "Button",
        "label":  "Help",
        "clickHandler":  null
    },
    {
        "form":  "FactionReminder.cs",
        "control":  "MyNoButton",
        "type":  "Button",
        "label":  "No",
        "clickHandler":  null
    },
    {
        "form":  "FactionReminder.cs",
        "control":  "MyYesButton",
        "type":  "Button",
        "label":  "Yes",
        "clickHandler":  null
    },
    {
        "form":  "GliderForm.cs",
        "control":  "AddFactionButton",
        "type":  "Button",
        "label":  "Add Faction",
        "clickHandler":  "AddFactionButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "AddWaypointButton",
        "type":  "Button",
        "label":  "Add Waypoint",
        "clickHandler":  "AddWaypointButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "ConfigButton",
        "type":  "Button",
        "label":  "Configure",
        "clickHandler":  "ConfigButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "EditProfileButton",
        "type":  "Button",
        "label":  "Edit Profile",
        "clickHandler":  "EditProfileButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "GlideButton",
        "type":  "Button",
        "label":  "Glide",
        "clickHandler":  "GlideButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "HideButton",
        "type":  "Button",
        "label":  "HideButton",
        "clickHandler":  "HideButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "KillButton",
        "type":  "Button",
        "label":  "1-Kill",
        "clickHandler":  "KillButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "LoadProfileButton",
        "type":  "Button",
        "label":  "Load Profile",
        "clickHandler":  "LoadProfileButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "MyHelpButton",
        "type":  "Button",
        "label":  "MyHelpButton",
        "clickHandler":  "MyHelpButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "NewProfileButton",
        "type":  "Button",
        "label":  "New Profile",
        "clickHandler":  "NewProfileButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "QuickLoadButton",
        "type":  "Button",
        "label":  "QuickLoadButton",
        "clickHandler":  "QuickLoadButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "SaveProfileButton",
        "type":  "Button",
        "label":  "Save Profile",
        "clickHandler":  "SaveProfileButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "ShrinkButton",
        "type":  "Button",
        "label":  "ShrinkButton",
        "clickHandler":  "ShrinkButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "StopButton",
        "type":  "Button",
        "label":  "Stop",
        "clickHandler":  "StopButton_Click"
    },
    {
        "form":  "GliderForm.cs",
        "control":  "WPTypeAuto_1",
        "type":  "RadioButton",
        "label":  "Automatic",
        "clickHandler":  null
    },
    {
        "form":  "GliderForm.cs",
        "control":  "WPTypeGhost_1",
        "type":  "RadioButton",
        "label":  "Ghost",
        "clickHandler":  null
    },
    {
        "form":  "GliderForm.cs",
        "control":  "WPTypeNormal_1",
        "type":  "RadioButton",
        "label":  "Normal",
        "clickHandler":  null
    },
    {
        "form":  "GliderForm.cs",
        "control":  "WPTypeVendor_1",
        "type":  "RadioButton",
        "label":  "Vendor",
        "clickHandler":  null
    },
    {
        "form":  "GliderWarning.cs",
        "control":  "MyNoButton",
        "type":  "Button",
        "label":  "No",
        "clickHandler":  "MyNoButton_Click"
    },
    {
        "form":  "GliderWarning.cs",
        "control":  "MyOKButton",
        "type":  "Button",
        "label":  "OK",
        "clickHandler":  "MyOKButton_Click"
    },
    {
        "form":  "GliderWarning.cs",
        "control":  "MyYesButton",
        "type":  "Button",
        "label":  "Yes",
        "clickHandler":  "MyYesButton_Click"
    },
    {
        "form":  "KeyEditor.cs",
        "control":  "MyCancelButton",
        "type":  "Button",
        "label":  "Cancel",
        "clickHandler":  "MyCancelButton_Click"
    },
    {
        "form":  "KeyEditor.cs",
        "control":  "MyHelpButton",
        "type":  "Button",
        "label":  "Help",
        "clickHandler":  "MyHelpButton_Click"
    },
    {
        "form":  "KeyEditor.cs",
        "control":  "MySaveButton",
        "type":  "Button",
        "label":  "Save",
        "clickHandler":  "MySaveButton_Click"
    },
    {
        "form":  "KeyEditor.cs",
        "control":  "PickButton",
        "type":  "Button",
        "label":  "Pick",
        "clickHandler":  "PickButton_Click"
    },
    {
        "form":  "LaunchpadReminder.cs",
        "control":  "MyOKButton",
        "type":  "Button",
        "label":  "OK",
        "clickHandler":  "MyOKButton_Click"
    },
    {
        "form":  "LaunchpadReminder_1.cs",
        "control":  "MyOKButton",
        "type":  "Button",
        "label":  "OK",
        "clickHandler":  "MyOKButton_Click"
    },
    {
        "form":  "ProfileProps.cs",
        "control":  "Circle",
        "type":  "RadioButton",
        "label":  "Wander circle",
        "clickHandler":  null
    },
    {
        "form":  "ProfileProps.cs",
        "control":  "ClearGhostWaypoints",
        "type":  "Button",
        "label":  "Clear",
        "clickHandler":  "ClearGhostWaypoints_Click"
    },
    {
        "form":  "ProfileProps.cs",
        "control":  "ClearVendorWaypoints",
        "type":  "Button",
        "label":  "Clear",
        "clickHandler":  "ClearVendorWaypoints_Click"
    },
    {
        "form":  "ProfileProps.cs",
        "control":  "ClearWaypoints",
        "type":  "Button",
        "label":  "Clear",
        "clickHandler":  "ClearWaypoints_Click"
    },
    {
        "form":  "ProfileProps.cs",
        "control":  "MyCancelButton",
        "type":  "Button",
        "label":  "Cancel",
        "clickHandler":  "MyCancelButton_Click"
    },
    {
        "form":  "ProfileProps.cs",
        "control":  "MyHelpButton",
        "type":  "Button",
        "label":  "\u0026Help",
        "clickHandler":  "MyHelpButton_Click"
    },
    {
        "form":  "ProfileProps.cs",
        "control":  "MyOKButton",
        "type":  "Button",
        "label":  "OK",
        "clickHandler":  "MyOKButton_Click"
    },
    {
        "form":  "ProfileProps.cs",
        "control":  "OutAndBack",
        "type":  "RadioButton",
        "label":  "Wander out-and-back",
        "clickHandler":  null
    },
    {
        "form":  "ProfileWizard.cs",
        "control":  "NextButton",
        "type":  "Button",
        "label":  "Next \u003e\u003e",
        "clickHandler":  "NextButton_Click"
    },
    {
        "form":  "ProfileWizard.cs",
        "control":  "PrevButton",
        "type":  "Button",
        "label":  "\u003c\u003c Previous",
        "clickHandler":  "PrevButton_Click"
    },
    {
        "form":  "RemindBar.cs",
        "control":  "MyNoButton",
        "type":  "Button",
        "label":  "No",
        "clickHandler":  "MyNoButton_Click"
    },
    {
        "form":  "RemindBar.cs",
        "control":  "MyYesButton",
        "type":  "Button",
        "label":  "Yes",
        "clickHandler":  "MyYesButton_Click"
    }
]
;

if (typeof window !== "undefined") {
    window.LEGACY_CONTROLS = LEGACY_CONTROLS;
}

