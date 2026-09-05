//-- package.rs ---------------------------------------------------------------------------------------------------------------------

use	serde::{ Deserialize, Serialize };
// no custom silo imports needed

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, Serialize, Deserialize)]
pub struct KernelDependency
{
    pub _Name:    String,
    pub _Version: String,
    pub _Url:     Option< String>,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug, Serialize, Deserialize)]
pub struct KernelExport
{
    pub _KernelName:   String,
    pub _Symbol:       String,
    pub _IsBehavioral: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a TOML-based manifest for distributing modular IP and kernels.
/// This matches standard EDA IP packaging formats (e.g. IP-XACT).
#[derive( Clone, Debug, Serialize, Deserialize)]
pub struct ModulePackage
{
    pub _PackageName:  String,
    pub _Version:      String,
    pub _Description:  String,
    pub _Authors:      Box< [String]>,
    pub _Dependencies: Box< [KernelDependency]>,
    pub _Exports:      Box< [KernelExport]>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ModulePackage
{
    /// Attempts to parse a package manifest from a TOML or JSON string
    pub fn	FromJson( jsonStr: &str) -> Result< Self, serde_json::Error>
    {
        return serde_json::from_str( jsonStr);
    }

    /// Serializes the package manifest to JSON
    pub fn	ToJson( &self) -> Result< String, serde_json::Error>
    {
        return serde_json::to_string_pretty( self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
pub mod _tests
{
    use	super::*;

    #[test]
    fn	test_module_package_serialization()
    {
        let  	jsonStr = r#"{
            "_PackageName": "CoreIP",
            "_Version": "1.0.0",
            "_Description": "Core simulation IP kernels",
            "_Authors": ["Kosh Engineering"],
            "_Dependencies": [],
            "_Exports": [
                {
                    "_KernelName": "BusAdder32",
                    "_Symbol": "BusAdder32Kernel",
                    "_IsBehavioral": false
                }
            ]
        }"#;

        let  	pkg = ModulePackage::FromJson( jsonStr).expect( "Failed to parse JSON manifest");
        assert_eq!( pkg._PackageName, "CoreIP");
        assert_eq!( pkg._Authors.len(), 1);
        assert_eq!( pkg._Exports.len(), 1);
        assert_eq!( pkg._Exports[0]._KernelName, "BusAdder32");
    }
}

