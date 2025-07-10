// --------------------- Macros for use in rust tests -------------------------
extern crate proc_macro;

#[derive(PartialEq, Clone, Copy)]
enum FmiVersion {
    Fmi2,
    Fmi3
}

const NUM_OF_FMI_VERSIONS: usize = 2;

impl FmiVersion {
    pub fn function_suffix_part(&self) -> std::string::String {
        match self {
            FmiVersion::Fmi2 => std::string::String::from("fmi2"),
            FmiVersion::Fmi3 => std::string::String::from("fmi3")
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum ProgrammingLanguage {
    CSharp,
    Java,
    Python
}

const NUM_OF_PROGRAMMING_LANGUAGES: usize = 3;

impl ProgrammingLanguage {
    pub fn function_suffix_part(&self) -> std::string::String {
        match self {
            ProgrammingLanguage::CSharp => std::string::String::from("csharp"),
            ProgrammingLanguage::Java => std::string::String::from("java"),
            ProgrammingLanguage::Python => std::string::String::from("python")
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum FmuPackaging {
    BareDirectory,
    Zipped
}

const NUM_OF_FMU_PACKGAGINGS: usize = 2;

impl FmuPackaging {
    pub fn function_suffix_part(&self) -> std::string::String {
        match self {
            FmuPackaging::BareDirectory => std::string::String::from("bare_directory"),
            FmuPackaging::Zipped => std::string::String::from("zipped")
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum FmuBackend {
    Blackbox,
    Distributed,
    Local
}

const NUM_OF_FMU_BACKENDS: usize = 3;

impl FmuBackend {
    pub fn function_suffix_part(&self) -> std::string::String {
        match self {
            FmuBackend::Blackbox => std::string::String::from("blackbox"),
            FmuBackend::Distributed => std::string::String::from("distributed"),
            FmuBackend::Local => std::string::String::from("local")
        }
    }
}

struct FmuVariant {
    pub version: FmiVersion,
    pub language: ProgrammingLanguage,
    pub packaging: FmuPackaging,
    pub backend: FmuBackend
}

impl FmuVariant {
    pub fn function_suffix(&self) -> std::string::String {
        std::format!(
            "_{}_{}_{}_{}",
            self.version.function_suffix_part(),
            self.language.function_suffix_part(),
            self.packaging.function_suffix_part(),
            self.backend.function_suffix_part()
        )
    }
}

#[derive(Default)]
struct FmuPossibilities {
    pub fmi2: bool,
    pub fmi3: bool,
    pub csharp: bool,
    pub java: bool,
    pub python: bool,
    pub bare_directory: bool,
    pub zipped: bool,
    pub blackbox: bool,
    pub distributed: bool,
    pub local: bool
}

impl FmuPossibilities {
    pub fn with_all_enabled() -> Self {
        FmuPossibilities {
            fmi2: true,
            fmi3: true,
            csharp: true,
            java: true,
            python: true,
            bare_directory: true,
            zipped: true,
            blackbox: true,
            distributed: true,
            local: true
        }
    }

    pub fn get_variations(&self) -> Vec<FmuVariant> {
        let mut versions: Vec<FmiVersion> = Vec::with_capacity(NUM_OF_FMI_VERSIONS);
        if self.fmi2 {versions.push(FmiVersion::Fmi2);}
        if self.fmi3 {versions.push(FmiVersion::Fmi3);}

        let mut languages: Vec<ProgrammingLanguage> = Vec::with_capacity(NUM_OF_PROGRAMMING_LANGUAGES);
        if self.csharp {languages.push(ProgrammingLanguage::CSharp);}
        if self.java {languages.push(ProgrammingLanguage::Java);}
        if self.python {languages.push(ProgrammingLanguage::Python);}

        let mut packagings: Vec<FmuPackaging> = Vec::with_capacity(NUM_OF_FMU_PACKGAGINGS);
        if self.bare_directory {packagings.push(FmuPackaging::BareDirectory);}
        if self.zipped {packagings.push(FmuPackaging::Zipped);}

        let mut backends: Vec<FmuBackend> = Vec::with_capacity(NUM_OF_FMU_BACKENDS);
        if self.blackbox {backends.push(FmuBackend::Blackbox);}
        if self.distributed {backends.push(FmuBackend::Distributed);}
        if self.local {backends.push(FmuBackend::Local);}
        
        let mut variations: Vec<FmuVariant> = Vec::with_capacity(
            NUM_OF_FMI_VERSIONS
            * NUM_OF_PROGRAMMING_LANGUAGES
            * NUM_OF_FMU_PACKGAGINGS
            * NUM_OF_FMU_BACKENDS
        );

        for &version in &versions {
            for &language in &languages {
                for &packaging in &packagings {
                    for &backend in &backends {
                        if backend == FmuBackend::Blackbox
                            && (
                                language != ProgrammingLanguage::Python
                                || packaging != FmuPackaging::BareDirectory
                            )
                        {
                            // Blackbox FMU's must be python based and not zipped.
                            continue;
                        }

                        variations.push(
                            FmuVariant {version, language, packaging, backend}
                        );
                    }
                }
            }
        }

        variations
    }
}

#[proc_macro_attribute]
pub fn for_each_fmu(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let original_function = syn::parse_macro_input!(item as syn::ItemFn);

    let functions = FmuPossibilities::with_all_enabled()
        .get_variations()
        .into_iter()
        .map(|variation| {
            let function_clone = original_function.clone();

            syn::ItemFn {
                sig: syn::Signature {
                    ident: syn::Ident::new(
                        &std::format!(
                            "{}{}",
                            function_clone.sig.ident,
                            variation.function_suffix()
                        ), 
                        function_clone.sig.ident.span()
                    ),
                    ..function_clone.sig
                },
                ..function_clone
            }
        });

    let expanded = quote::quote! {
        #(#functions)*
    };

    proc_macro::TokenStream::from(expanded)
}

