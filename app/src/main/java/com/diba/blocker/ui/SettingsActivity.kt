package com.diba.blocker.ui

import android.os.Bundle
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.diba.blocker.R
import com.google.android.material.button.MaterialButton

class SettingsActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_settings)

        findViewById<TextView>(R.id.tvAppVersion).text = "Version 1.0.0 (Rust Core)"
        findViewById<MaterialButton>(R.id.btnBack).setOnClickListener {
            finish()
        }
    }
}
