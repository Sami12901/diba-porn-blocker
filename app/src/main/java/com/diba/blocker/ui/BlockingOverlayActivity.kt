package com.diba.blocker.ui

import android.content.Intent
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.diba.blocker.R

class BlockingOverlayActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_blocking_overlay)

        val reason = intent.getStringExtra(EXTRA_REASON) ?: getString(R.string.overlay_blocked_title)
        val term = intent.getStringExtra(EXTRA_TERM) ?: ""

        val tvReason = findViewById<TextView>(R.id.tvBlockReason)
        val tvTerm = findViewById<TextView>(R.id.tvBlockTerm)
        val btnSafety = findViewById<Button>(R.id.btnLeaveSafety)

        tvReason.text = reason
        tvTerm.text = if (term.isNotEmpty()) "Target: $term" else ""

        btnSafety.setOnClickListener {
            returnToHome()
        }
    }

    override fun onBackPressed() {
        super.onBackPressed()
        returnToHome()
    }

    private fun returnToHome() {
        val homeIntent = Intent(Intent.ACTION_MAIN).apply {
            addCategory(Intent.CATEGORY_HOME)
            flags = Intent.FLAG_ACTIVITY_NEW_TASK
        }
        startActivity(homeIntent)
        finish()
    }

    companion object {
        const val EXTRA_REASON = "extra_reason"
        const val EXTRA_TERM = "extra_term"
    }
}
