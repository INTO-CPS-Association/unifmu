extern crate proc_macro;

// --------------------- Macros for use in rust tests -------------------------

/// Replaces the decorated test function with duplicates for each variation of
/// FMU that the UniFMU tool can generate.
/// 
/// Each duplicate will have a unique function name based on the FMU variation,
/// all instances of WildFmu{} in the duplicate will be replaced with a call to
/// get_clone() for the relevant Fmu struct, and extra attribute macros will be
/// added to the duplicate if needed for proper test execution.
/// 
/// Adding either the "include:" or "exclude:" attribute to the macro, followed
/// by a comma seperated list of traits, will limit the duplicates to
/// variations with or without those traits respectively.
/// 
/// For example decorating a function with the "include:" attribute
/// ```
/// #[unifmu_macros::for_each_fmu(include: fmi2, python, bare_directory)]
/// #[test]
/// fn some_function() {
///     let _dummy_fmu = WildFmu{};
/// }
/// ```
/// will expand it to
/// ```
/// #[test]
/// #[serial_test::parallel]
/// fn some_function_fmi2_python_bare_directory_local() {
///     let _dummy_fmu = BasicFmu::get_clone(
///         FmiVersion::Fmi2,
///         FmuBackendImplementationLanguage::Python,
///     );
/// }
/// 
/// #[test]
/// #[serial_test::parallel]
/// fn some_function_fmi2_python_bare_directory_distributed() {
///     let _dummy_fmu = DistributedFmu::get_clone(
///         FmiVersion::Fmi2,
///         FmuBackendImplementationLanguage::Python,
///     );
/// }
/// 
/// #[test]
/// #[serial_test::parallel]
/// fn some_function_fmi2_python_bare_directory_blackbox() {
///     let _dummy_fmu = BlackboxDistributedFmu::get_clone(
///         FmiVersion::Fmi2,
///         FmuBackendImplementationLanguage::Python,
///     );
/// }
/// ```
/// 
/// while decorating the same function with the "exclude:" attribute
/// ```
/// #[unifmu_macros::for_each_fmu(exclude: fmi2, python, bare_directory)]
/// #[test]
/// fn some_function() {
///     let _dummy_fmu = WildFmu{};
/// }
/// ```
/// will expand it to
/// ```
/// #[test]
/// #[serial_test::parallel]
/// fn some_function_fmi3_csharp_zipped_local() {
///     let _dummy_fmu = BasicFmu::get_clone(
///         FmiVersion::Fmi3,
///         FmuBackendImplementationLanguage::CSharp,
///     );
/// }
/// 
/// #[test]
/// #[serial_test::parallel]
/// fn some_function_fmi3_csharp_zipped_distributed() {
///     let _dummy_fmu = DistributedFmu::get_clone(
///         FmiVersion::Fmi3,
///         FmuBackendImplementationLanguage::CSharp,
///     );
/// }
/// 
/// #[test]
/// #[serial_test::serial]
/// fn some_function_fmi3_java_zipped_local() {
///     let _dummy_fmu = BasicFmu::get_clone(
///         FmiVersion::Fmi3,
///         FmuBackendImplementationLanguage::Java,
///     );
/// }
/// 
/// #[test]
/// #[serial_test::serial]
/// fn some_function_fmi3_java_zipped_distributed() {
///     let _dummy_fmu = DistributedFmu::get_clone(
///         FmiVersion::Fmi3,
///         FmuBackendImplementationLanguage::Java,
///     );
/// }
/// ```
/// 
/// Note that functions for non-existing variations (like a FMI3, Java based,
/// zipped, blackbox FMU) will not be generated.
/// 
/// The possible traits are:
///  - fmi2
///  - fmi3
///  - csharp
///  - java
///  - python
///  - bare_directory
///  - zipped
///  - local
///  - distributed
///  - blackbox
#[proc_macro_attribute]
pub fn for_each_fmu(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream
) -> proc_macro::TokenStream {
    let original_function = syn::parse_macro_input!(item as syn::ItemFn);
    let possibilities = syn::parse_macro_input!(attr as FmuPossibilities);

    let marker_ident = syn::Ident::new(
        "WildFmu",
        proc_macro2::Span::call_site()
    );

    // Generate the duplicate functions
    let functions = possibilities
        .get_variations()
        .into_iter()
        .map(|variant| {
            let mut function_clone = original_function.clone();

            // Ensure that all other attribute macros present on the original
            // function are included on each duplicate function, along with
            // any  variation dependent extra attributes.
            let mut new_attrs: Vec<syn::Attribute> = Vec::with_capacity(
                function_clone.attrs.capacity() + 1
            );
            new_attrs.append(&mut function_clone.attrs);
            new_attrs.push(variant.extra_attribute());

            let variant_struct = variant.struct_type();
            let variant_version = variant.version_type();
            let variant_language = variant.language_type();

            // Replace each instance of assigning a WildFmu{} to a new variable
            // with assigning the result of a call to the correct version of
            // get_clone() with the the FMU variations FMI version and
            // programming language.
            let new_block = syn::Block {
                brace_token: function_clone.block.brace_token,
                stmts: function_clone.block.stmts.iter()
                    .map(|stmt| {
                        if let syn::Stmt::Local(local_stmt) = stmt {
                            if let std::option::Option::Some(local_init) = &local_stmt.init {
                                if let syn::Expr::Struct(struct_expr) = local_init.expr.as_ref() {
                                    if struct_expr.path.is_ident(&marker_ident) {
                                        let variable_pattern = &local_stmt.pat;
                                        let new_stmt: syn::Stmt = syn::parse(
                                            quote::quote! {
                                                let #variable_pattern = #variant_struct::get_clone(
                                                    &#variant_version,
                                                    &#variant_language
                                                );
                                            }.into()
                                        ).expect("macro should be able to expand DummyFmu into actual Fmu variant");
                                        return new_stmt
                                    }
                                }
                            }
                        };
                        stmt.to_owned()
                    })
                    .collect()
            };

            // Assemble the new duplicate function from the new identity (name),
            // the new function body, the new attributes, and everything else
            // from the original function.
            syn::ItemFn {
                attrs: new_attrs,
                sig: syn::Signature {
                    ident: syn::Ident::new(
                        &std::format!(
                            "{}{}",
                            function_clone.sig.ident,
                            variant.function_suffix()
                        ), 
                        function_clone.sig.ident.span()
                    ),
                    ..function_clone.sig
                },
                block: std::boxed::Box::new(new_block),
                ..function_clone
            }
        });

    let expanded = quote::quote! {
        #(#functions)*

    };

    proc_macro::TokenStream::from(expanded)
}

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

    pub fn type_path(&self) -> proc_macro2::TokenStream {
        let ident_string = match self {
            FmiVersion::Fmi2 => "Fmi2",
            FmiVersion::Fmi3 => "Fmi3"
        };

        let path_segment = syn::PathSegment {
            ident: syn::Ident::new(ident_string, proc_macro2::Span::call_site()),
            arguments: syn::PathArguments::None
        };

        quote::quote! {common::FmiVersion::#path_segment}
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

    pub fn type_path(&self) -> proc_macro2::TokenStream {
        let ident_string = match self {
            ProgrammingLanguage::CSharp => "CSharp",
            ProgrammingLanguage::Java => "Java",
            ProgrammingLanguage::Python => "Python"
        };

        let path_segment = syn::PathSegment {
            ident: syn::Ident::new(ident_string, proc_macro2::Span::call_site()),
            arguments: syn::PathArguments::None
        };

        quote::quote! {common::FmuBackendImplementationLanguage::#path_segment}
    }

    pub fn processing_attribute(&self) -> syn::Attribute {
        match self {
            ProgrammingLanguage::Java => syn::parse_quote!{#[serial_test::serial]},
            _ => syn::parse_quote!{#[serial_test::parallel]}
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

    pub fn struct_type_part(&self) -> std::string::String {
        match self {
            FmuPackaging::BareDirectory => std::string::String::from(""),
            FmuPackaging::Zipped => std::string::String::from("Zipped")
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

    pub fn struct_type_part(&self) -> std::string::String {
        match self {
            FmuBackend::Blackbox => std::string::String::from("BlackboxDistributed"),
            FmuBackend::Distributed => std::string::String::from("Distributed"),
            FmuBackend::Local => std::string::String::from("Local")
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

    pub fn struct_type(&self) -> proc_macro2::TokenStream {
        let struct_path_segment = syn::PathSegment {
            ident: syn::Ident::new(
                &std::format!(
                    "{}{}Fmu",
                    self.packaging.struct_type_part(),
                    self.backend.struct_type_part()
                ),
                proc_macro2::Span::call_site()
            ),
            arguments: syn::PathArguments::None
        };

        quote::quote! {common::#struct_path_segment}
    }

    pub fn version_type(&self) -> proc_macro2::TokenStream {
        self.version.type_path()
    }

    pub fn language_type(&self) -> proc_macro2::TokenStream {
        self.language.type_path()
    }

    pub fn extra_attribute(&self) -> syn::Attribute {
        self.language.processing_attribute()
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

    pub fn with_all_disabled() -> Self {
        FmuPossibilities {
            fmi2: false,
            fmi3: false,
            csharp: false,
            java: false,
            python: false,
            bare_directory: false,
            zipped: false,
            blackbox: false,
            distributed: false,
            local: false
        }
    }

    pub fn enable_possibility(&mut self, possibility_name: &str) -> Result<(),()> {
        self.update_possibility(possibility_name, true)
    }

    pub fn disable_possibility(&mut self, possibility_name: &str) -> Result<(),()> {
        self.update_possibility(possibility_name, false)
    }

    pub fn update_possibility(&mut self, possibility_name: &str, new_value: bool) -> Result<(),()> {
        match possibility_name {
            "fmi2" => self.fmi2 = new_value,
            "fmi3" => self.fmi3 = new_value,
            "csharp" => self.csharp = new_value,
            "jave" => self.java = new_value,
            "python" => self.python = new_value,
            "bare_directory" => self.bare_directory = new_value,
            "zipped" => self.zipped = new_value,
            "blackbox" => self.blackbox = new_value,
            "distributed" => self.distributed = new_value,
            "local" => self.local = new_value,
            _ => return std::result::Result::Err(())
        }
        std::result::Result::Ok(())
    }

    pub fn enable_all_fully_disabled_possibility_groups(&mut self) {
        if !self.fmi2 && !self.fmi3 {
            self.fmi2 = true;
            self.fmi3 = true;
        }
        if !self.csharp && !self.java && !self.python {
            self.csharp = true;
            self.java = true;
            self.python = true;
        }
        if !self.bare_directory && !self.zipped {
            self.bare_directory = true;
            self.zipped = true;
        }
        if !self.blackbox && !self.distributed && !self.local {
            self.blackbox = true;
            self.distributed = true;
            self.local = true;
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
                            // Blackbox FMUs must be python based and not zipped.
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

impl syn::parse::Parse for FmuPossibilities {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            let initial_identity: syn::Ident = input.parse()?;
            let _: syn::Token![:] = input.parse()?;
        
            match initial_identity.to_string().as_str() {
                "include" => {
                    let mut fmu_possibilities = FmuPossibilities::with_all_disabled();

                    while !input.is_empty() {
                        let included: syn::Ident = input.parse()?;

                        fmu_possibilities.enable_possibility(&included.to_string())
                            .map_err(|_| syn::Error::new(
                                included.span(),
                                "the excluded possibility is not recognized"
                            ))?;

                        if input.peek(syn::Token![,]) {
                            let _: syn::Token![,] = input.parse()?;
                        }
                    }
                    fmu_possibilities.enable_all_fully_disabled_possibility_groups();

                    std::result::Result::Ok(fmu_possibilities)
                }
                "exclude" => {
                    let mut fmu_possibilities = FmuPossibilities::with_all_enabled();

                    while !input.is_empty() {
                        let excluded: syn::Ident = input.parse()?;

                        fmu_possibilities.disable_possibility(&excluded.to_string())
                            .map_err(|_| syn::Error::new(
                                excluded.span(),
                                "the excluded possibility is not recognized"
                            ))?;

                        if input.peek(syn::Token![,]) {
                            let _: syn::Token![,] = input.parse()?;
                        }
                    }
                    std::result::Result::Ok(fmu_possibilities)
                }
                _ => {
                    std::result::Result::Err(syn::Error::new(
                        initial_identity.span(),
                        "first attribute must be either 'include' or 'exclude'"
                    ))
                }
            }
        } else {
            std::result::Result::Ok(FmuPossibilities::with_all_enabled())
        }
    }
}