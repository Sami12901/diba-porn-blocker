package com.diba.blocker.service

import android.accessibilityservice.AccessibilityService
import android.content.Intent
import android.os.SystemClock
import android.util.Log
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import com.diba.blocker.core.NativeBridge
import com.diba.blocker.ui.BlockingOverlayActivity

class DibaAccessibilityService : AccessibilityService() {

    private var lastBlockTimestamp: Long = 0
    private val blockDebounceMs: Long = 1000

    private val browserPackages = setOf(
        "com.android.chrome",
        "com.brave.browser",
        "com.microsoft.emmx",
        "org.mozilla.firefox",
        "com.sec.android.app.sbrowser",
        "com.opera.browser",
        "com.opera.mini.native",
        "com.duckduckgo.mobile.android",
        "com.vivaldi.browser",
        "com.UCMobile.intl",
        "org.torproject.torbrowser",
        "com.google.android.youtube",
        "com.google.android.googlequicksearchbox"
    )

    private val urlViewIds = setOf(
        "url_bar",
        "search_box",
        "search_src_text",
        "location_bar_edit_text",
        "address_bar",
        "url_field",
        "mozac_browser_toolbar_url_view",
        "toolbar",
        "omnibox",
        "search_edit_frame"
    )

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        if (event == null) return

        val pkgName = event.packageName?.toString() ?: return
        if (!browserPackages.contains(pkgName)) return

        val rootNode = rootInActiveWindow ?: return

        try {
            inspectNodeTree(rootNode, pkgName)
        } catch (e: Exception) {
            Log.e(TAG, "Error inspecting node hierarchy: ${e.message}")
        } finally {
            rootNode.recycle()
        }
    }

    private fun inspectNodeTree(node: AccessibilityNodeInfo?, pkgName: String) {
        if (node == null) return

        // 1. Check if node is an address bar or contains text
        val viewId = node.viewIdResourceName?.lowercase() ?: ""
        val text = node.text?.toString() ?: ""
        val contentDesc = node.contentDescription?.toString() ?: ""

        val isAddressBar = urlViewIds.any { viewId.contains(it) } ||
                viewId.contains("url") || viewId.contains("address")

        if (isAddressBar && text.isNotEmpty()) {
            val decision = NativeBridge.evaluateUrl(text)
            if (decision.isBlocked) {
                triggerBlock(decision.reason ?: "Adult Content", decision.matchedTerm ?: text)
                return
            }
        }

        // 2. Scan visible text if it contains explicit phrases
        if (text.length in 3..256) {
            // Check if it looks like a URL
            if (text.contains(".") && !text.contains(" ") && (text.startsWith("http") || text.startsWith("www") || text.endsWith(".com") || text.endsWith(".net") || text.endsWith(".org") || text.endsWith(".xxx"))) {
                val decision = NativeBridge.evaluateUrl(text)
                if (decision.isBlocked) {
                    triggerBlock(decision.reason ?: "Adult Domain", decision.matchedTerm ?: text)
                    return
                }
            } else {
                val textDecision = NativeBridge.evaluateText(text)
                if (textDecision.isBlocked) {
                    triggerBlock(textDecision.reason ?: "Prohibited Keyword", textDecision.matchedTerm ?: text)
                    return
                }
            }
        }

        // 3. Scan content description (some browsers put URL in contentDescription)
        if (contentDesc.isNotEmpty() && contentDesc.contains(".")) {
            val descDecision = NativeBridge.evaluateUrl(contentDesc)
            if (descDecision.isBlocked) {
                triggerBlock(descDecision.reason ?: "Adult Content", descDecision.matchedTerm ?: contentDesc)
                return
            }
        }

        // Recursively inspect children
        val childCount = node.childCount
        for (i in 0 until childCount) {
            val child = node.getChild(i)
            if (child != null) {
                inspectNodeTree(child, pkgName)
                child.recycle()
            }
        }
    }

    private fun triggerBlock(reason: String, matchedTerm: String) {
        val now = SystemClock.elapsedRealtime()
        if (now - lastBlockTimestamp < blockDebounceMs) {
            return
        }
        lastBlockTimestamp = now

        Log.w(TAG, "Diba BLOCKED: $reason ($matchedTerm)")

        // Instantly kick the user out of the forbidden content to Home Screen
        performGlobalAction(GLOBAL_ACTION_HOME)

        // Launch full-screen protective barrier overlay
        val overlayIntent = Intent(this, BlockingOverlayActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or
                    Intent.FLAG_ACTIVITY_CLEAR_TOP or
                    Intent.FLAG_ACTIVITY_EXCLUDE_FROM_RECENTS
            putExtra(BlockingOverlayActivity.EXTRA_REASON, reason)
            putExtra(BlockingOverlayActivity.EXTRA_TERM, matchedTerm)
        }
        startActivity(overlayIntent)
    }

    override fun onInterrupt() {
        Log.i(TAG, "Diba Accessibility Service interrupted")
    }

    companion object {
        private const val TAG = "DibaAccessService"
    }
}
