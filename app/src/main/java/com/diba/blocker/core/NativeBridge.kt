package com.diba.blocker.core

import android.util.Log
import org.json.JSONObject

data class BlockDecision(
    val isBlocked: Boolean,
    val reason: String?,
    val matchedTerm: String?
)

data class EngineStats(
    val totalScanned: Long,
    val totalBlocked: Long,
    val rulesCount: Int,
    val isActive: Boolean,
    val hasPin: Boolean
)

object NativeBridge {
    private const val TAG = "DibaNativeBridge"
    private var isLoaded = false

    init {
        try {
            System.loadLibrary("diba_core")
            isLoaded = true
            Log.d(TAG, "libdiba_core.so loaded successfully")
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "Could not load libdiba_core: ${e.message}")
        }
    }

    fun init(): Boolean {
        return if (isLoaded) initEngine() else false
    }

    fun evaluateUrl(url: String): BlockDecision {
        if (!isLoaded) return BlockDecision(false, null, null)
        val jsonStr = checkUrl(url)
        return parseDecision(jsonStr)
    }

    fun evaluateText(text: String): BlockDecision {
        if (!isLoaded) return BlockDecision(false, null, null)
        val jsonStr = checkText(text)
        return parseDecision(jsonStr)
    }

    fun fetchStats(): EngineStats {
        if (!isLoaded) return EngineStats(0, 0, 0, false, false)
        val jsonStr = getEngineStats()
        return try {
            val obj = JSONObject(jsonStr)
            EngineStats(
                totalScanned = obj.optLong("total_scanned", 0L),
                totalBlocked = obj.optLong("total_blocked", 0L),
                rulesCount = obj.optInt("rules_count", 0),
                isActive = obj.optBoolean("is_active", true),
                hasPin = obj.optBoolean("has_pin", false)
            )
        } catch (e: Exception) {
            EngineStats(0, 0, 0, false, false)
        }
    }

    private fun parseDecision(jsonStr: String): BlockDecision {
        return try {
            val obj = JSONObject(jsonStr)
            val isBlocked = obj.optBoolean("is_blocked", false)
            val reason = if (obj.has("reason") && !obj.isNull("reason")) obj.getString("reason") else null
            val matched = if (obj.has("matched_term") && !obj.isNull("matched_term")) obj.getString("matched_term") else null
            BlockDecision(isBlocked, reason, matched)
        } catch (e: Exception) {
            BlockDecision(false, null, null)
        }
    }

    // Native JNI Declarations implemented in libdiba_core.so
    @JvmStatic external fun initEngine(): Boolean
    @JvmStatic external fun checkUrl(url: String): String
    @JvmStatic external fun checkText(text: String): String
    @JvmStatic external fun addBlockedDomain(domain: String): Boolean
    @JvmStatic external fun addWhitelistedDomain(domain: String): Boolean
    @JvmStatic external fun addBlockedKeyword(keyword: String): Boolean
    @JvmStatic external fun setPin(pin: String): Boolean
    @JvmStatic external fun verifyPin(pin: String): Boolean
    @JvmStatic external fun setProtectionActive(active: Boolean): Boolean
    @JvmStatic external fun getEngineStats(): String
}
