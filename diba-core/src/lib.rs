pub mod blocklist;
pub mod keywords;
pub mod security;
pub mod engine;

use engine::{BlockDecision, DibaEngine, EngineStats};
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::sync::{Mutex, OnceLock};

static GLOBAL_ENGINE: OnceLock<Mutex<DibaEngine>> = OnceLock::new();

fn get_engine() -> &'static Mutex<DibaEngine> {
    GLOBAL_ENGINE.get_or_init(|| Mutex::new(DibaEngine::new()))
}

// ---------------------------------------------------------------------------
// JNI Exported Functions for Android
// Package: com.diba.blocker.core.NativeBridge
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_initEngine(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let _ = get_engine();
    JNI_TRUE
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_checkUrl(
    mut env: JNIEnv,
    _class: JClass,
    input_url: JString,
) -> jstring {
    let url_str: String = match env.get_string(&input_url) {
        Ok(s) => s.into(),
        Err(_) => String::new(),
    };

    let decision = if let Ok(mut engine) = get_engine().lock() {
        engine.evaluate_url(&url_str)
    } else {
        BlockDecision {
            is_blocked: false,
            reason: None,
            matched_term: None,
        }
    };

    let json_output = serde_json::to_string(&decision).unwrap_or_else(|_| "{}".to_string());
    match env.new_string(json_output) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_checkText(
    mut env: JNIEnv,
    _class: JClass,
    input_text: JString,
) -> jstring {
    let text_str: String = match env.get_string(&input_text) {
        Ok(s) => s.into(),
        Err(_) => String::new(),
    };

    let decision = if let Ok(mut engine) = get_engine().lock() {
        engine.evaluate_text(&text_str)
    } else {
        BlockDecision {
            is_blocked: false,
            reason: None,
            matched_term: None,
        }
    };

    let json_output = serde_json::to_string(&decision).unwrap_or_else(|_| "{}".to_string());
    match env.new_string(json_output) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_addBlockedDomain(
    mut env: JNIEnv,
    _class: JClass,
    domain: JString,
) -> jboolean {
    if let Ok(s) = env.get_string(&domain) {
        let domain_str: String = s.into();
        if let Ok(mut engine) = get_engine().lock() {
            engine.add_blocked_domain(&domain_str);
            return JNI_TRUE;
        }
    }
    JNI_FALSE
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_addWhitelistedDomain(
    mut env: JNIEnv,
    _class: JClass,
    domain: JString,
) -> jboolean {
    if let Ok(s) = env.get_string(&domain) {
        let domain_str: String = s.into();
        if let Ok(mut engine) = get_engine().lock() {
            engine.add_whitelisted_domain(&domain_str);
            return JNI_TRUE;
        }
    }
    JNI_FALSE
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_addBlockedKeyword(
    mut env: JNIEnv,
    _class: JClass,
    keyword: JString,
) -> jboolean {
    if let Ok(s) = env.get_string(&keyword) {
        let kw_str: String = s.into();
        if let Ok(mut engine) = get_engine().lock() {
            engine.add_blocked_keyword(&kw_str);
            return JNI_TRUE;
        }
    }
    JNI_FALSE
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_setPin(
    mut env: JNIEnv,
    _class: JClass,
    pin: JString,
) -> jboolean {
    if let Ok(s) = env.get_string(&pin) {
        let pin_str: String = s.into();
        if let Ok(mut engine) = get_engine().lock() {
            return if engine.set_pin(&pin_str) { JNI_TRUE } else { JNI_FALSE };
        }
    }
    JNI_FALSE
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_verifyPin(
    mut env: JNIEnv,
    _class: JClass,
    pin: JString,
) -> jboolean {
    if let Ok(s) = env.get_string(&pin) {
        let pin_str: String = s.into();
        if let Ok(engine) = get_engine().lock() {
            return if engine.verify_pin(&pin_str) { JNI_TRUE } else { JNI_FALSE };
        }
    }
    JNI_FALSE
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_setProtectionActive(
    _env: JNIEnv,
    _class: JClass,
    active: jboolean,
) -> jboolean {
    if let Ok(mut engine) = get_engine().lock() {
        engine.set_protection_active(active == JNI_TRUE);
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diba_blocker_core_NativeBridge_getEngineStats(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let stats = if let Ok(engine) = get_engine().lock() {
        engine.get_stats()
    } else {
        EngineStats::default()
    };

    let json_output = serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string());
    match env.new_string(json_output) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_blocking() {
        let mut engine = DibaEngine::new();

        // Exact match
        let res = engine.evaluate_url("https://www.pornhub.com/video");
        assert!(res.is_blocked, "Should block pornhub");

        // Subdomain match
        let res2 = engine.evaluate_url("https://beta.xvideos.com");
        assert!(res2.is_blocked, "Should block subdomain of xvideos");

        // Non-blocked site
        let res3 = engine.evaluate_url("https://www.wikipedia.org");
        assert!(!res3.is_blocked, "Should allow wikipedia");
    }

    #[test]
    fn test_whitelist_override() {
        let mut engine = DibaEngine::new();
        engine.add_whitelisted_domain("pornhub.com");
        let res = engine.evaluate_url("https://www.pornhub.com");
        assert!(!res.is_blocked, "Whitelisted domain should be permitted");
    }

    #[test]
    fn test_search_query_blocking() {
        let mut engine = DibaEngine::new();

        let res = engine.evaluate_url("https://www.google.com/search?q=free+porn+videos");
        assert!(res.is_blocked, "Should block google search containing 'porn'");

        let res2 = engine.evaluate_url("https://www.youtube.com/results?search_query=hot+xxx+clips");
        assert!(res2.is_blocked, "Should block youtube search for xxx");

        let res3 = engine.evaluate_url("https://www.google.com/search?q=how+to+learn+rust+programming");
        assert!(!res3.is_blocked, "Should allow benign searches");
    }

    #[test]
    fn test_leetspeak_and_normalization() {
        let mut engine = DibaEngine::new();

        let res = engine.evaluate_text("Check this p.0.r.n website");
        assert!(res.is_blocked, "Should detect obfuscated p.0.r.n");

        let res2 = engine.evaluate_text("watch s3x clip");
        assert!(res2.is_blocked, "Should detect s3x");
    }

    #[test]
    fn test_pin_and_security() {
        let mut engine = DibaEngine::new();
        assert!(engine.set_pin("1234"));
        assert!(engine.verify_pin("1234"));
        assert!(!engine.verify_pin("9999"));
    }
}
