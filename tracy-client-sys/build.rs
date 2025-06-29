use std::io::Write;

macro_rules! docs_rs {
    () => {
        option_env!("DOCS_RS") == Some("1")
    };
}

fn link_dependencies() {
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("linux" | "android") => println!("cargo:rustc-link-lib=dl"),
        Ok("freebsd" | "dragonfly") => println!("cargo:rustc-link-lib=c"),
        Ok("windows") => println!("cargo:rustc-link-lib=user32"),
        Ok(_) => {}
        Err(e) => {
            writeln!(::std::io::stderr(), "Unable to get target_os=`{e}`!")
                .expect("could not report the error");
            ::std::process::exit(0xfd);
        }
    }
    if std::env::var_os("CARGO_FEATURE_DEBUGINFOD").is_some() && !docs_rs!() {
        println!("cargo:rustc-link-lib=debuginfod");
    }
}

fn set_feature_defines(mut c: cc::Build) -> cc::Build {
    if std::env::var_os("CARGO_FEATURE_ENABLE").is_some() {
        c.define("TRACY_ENABLE", None);
    }
    if std::env::var_os("CARGO_FEATURE_TIMER_FALLBACK").is_some() {
        c.define("TRACY_TIMER_FALLBACK", None);
    }
    if std::env::var_os("CARGO_FEATURE_ONDEMAND").is_some() {
        c.define("TRACY_ON_DEMAND", None);
    }
    if std::env::var_os("CARGO_FEATURE_ONLY_LOCALHOST").is_some() {
        c.define("TRACY_ONLY_LOCALHOST", None);
    }
    if std::env::var_os("CARGO_FEATURE_ONLY_IPV4").is_some() {
        c.define("TRACY_ONLY_IPV4", None);
    }
    if std::env::var_os("CARGO_FEATURE_FIBERS").is_some() {
        c.define("TRACY_FIBERS", None);
    }
    if std::env::var_os("CARGO_FEATURE_MANUAL_LIFETIME").is_some() {
        c.define("TRACY_MANUAL_LIFETIME", None);
    }
    if std::env::var_os("CARGO_FEATURE_DELAYED_INIT").is_some() {
        c.define("TRACY_DELAYED_INIT", None);
    }
    if std::env::var_os("CARGO_FEATURE_FLUSH_ON_EXIT").is_some() {
        c.define("TRACY_NO_EXIT", None);
    }
    if std::env::var_os("CARGO_FEATURE_DEMANGLE").is_some() {
        c.define("TRACY_DEMANGLE", None);
    }
    if std::env::var_os("CARGO_FEATURE_DEBUGINFOD").is_some() && !docs_rs!() {
        c.define("TRACY_DEBUGINFOD", None);
    }

    // Note: these are inversed and check for `is_none`!
    if std::env::var_os("CARGO_FEATURE_SYSTEM_TRACING").is_none() {
        c.define("TRACY_NO_SYSTEM_TRACING", None);
    }
    if std::env::var_os("CARGO_FEATURE_CONTEXT_SWITCH_TRACING").is_none() {
        c.define("TRACY_NO_CONTEXT_SWITCH", None);
    }
    if std::env::var_os("CARGO_FEATURE_SAMPLING").is_none() {
        c.define("TRACY_NO_SAMPLING", None);
    }
    if std::env::var_os("CARGO_FEATURE_CODE_TRANSFER").is_none() {
        c.define("TRACY_NO_CODE_TRANSFER", None);
    }
    if std::env::var_os("CARGO_FEATURE_BROADCAST").is_none() {
        c.define("TRACY_NO_BROADCAST", None);
    }
    if std::env::var_os("CARGO_FEATURE_CALLSTACK_INLINES").is_none() {
        c.define("TRACY_NO_CALLSTACK_INLINES", None);
    }
    if std::env::var_os("CARGO_FEATURE_VERIFY").is_none() {
        c.define("TRACY_NO_VERIFY", None);
    }
    if std::env::var_os("CARGO_FEATURE_CRASH_HANDLER").is_none() {
        c.define("TRACY_NO_CRASH_HANDLER", None);
    }
    c
}

fn build_tracy_client() {
    if std::env::var_os("CARGO_FEATURE_ENABLE").is_some() {
        let mut builder = set_feature_defines(cc::Build::new());

        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
            // Used for synchronizing access to the `dbghelp.dll` symbol helper.
            // See the `dbghelp` module for more information.
            builder.define("TRACY_DBGHELP_LOCK", "RustBacktraceMutex");
        }

        let _ = builder
            .file("tracy/TracyClient.cpp")
            .cargo_warnings(true)
            .cargo_debug(true)
            .cargo_metadata(true)
            .cargo_output(true)
            .debug(true)
            .cpp_link_stdlib("stdc++")
            .cpp(true);
        if let Ok(tool) = builder.try_get_compiler() {
            if tool.is_like_gnu() || tool.is_like_clang() {
                // https://github.com/rust-lang/cc-rs/issues/855
                builder.flag("-std=c++11");

                // Special handling for MinGW to avoid stack smashing
                if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
                    println!("cargo:warning=Applying MinGW-specific flags");
                    builder.flag("-fno-stack-protector");
                    // Explicitly define these to avoid problematic features
                    builder.define("TRACY_NO_SAMPLING", None);
                    builder.define("TRACY_NO_VERIFY", None);
                    builder.define("TRACY_NO_CRASH_HANDLER", None);
                }
            }
            if tool.is_like_msvc() {
                builder.flag("/MT"); // Static linking for MSVC
            }
        }
        let _ = builder.try_flags_from_environment("TRACY_CLIENT_SYS_CXXFLAGS");
        builder.compile("libtracy-client.a");
        link_dependencies();
    }
}

fn main() {
    if let Ok(lib) = std::env::var("TRACY_CLIENT_LIB") {
        if let Ok(lib_path) = std::env::var("TRACY_CLIENT_LIB_PATH") {
            println!("cargo:rustc-link-search=native={lib_path}");
        }
        let kind = std::env::var_os("TRACY_CLIENT_STATIC");
        let mode = if kind.is_none() || kind.as_deref() == Some(std::ffi::OsStr::new("0")) {
            "dylib"
        } else {
            link_dependencies();
            "static"
        };
        println!("cargo:rustc-link-lib={mode}={lib}");
    } else {
        build_tracy_client();
    }
}
