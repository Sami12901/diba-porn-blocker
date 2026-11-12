package com.diba.blocker

import android.app.Application
import android.util.Log
import com.diba.blocker.core.NativeBridge

class DibaApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        try {
            NativeBridge.init()
            Log.i(TAG, "Diba Rust core engine initialized successfully")
        } catch (e: Throwable) {
            Log.e(TAG, "Failed to load Diba Rust core: ${e.message}", e)
        }
    }

    companion object {
        private const val TAG = "DibaApplication"
    }
}
