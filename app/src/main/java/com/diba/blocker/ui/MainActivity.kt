package com.diba.blocker.ui

import android.app.admin.DevicePolicyManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.Settings
import android.view.LayoutInflater
import android.widget.*
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import com.diba.blocker.R
import com.diba.blocker.core.NativeBridge
import com.diba.blocker.receiver.DibaDeviceAdminReceiver
import com.google.android.material.button.MaterialButton
import com.google.android.material.materialswitch.MaterialSwitch

class MainActivity : AppCompatActivity() {

    private lateinit var tvStatusTitle: TextView
    private lateinit var tvStatusDesc: TextView
    private lateinit var switchProtection: MaterialSwitch
    private lateinit var tvScannedCount: TextView
    private lateinit var tvBlockedCount: TextView
    private lateinit var tvRulesCount: TextView

    private lateinit var btnEnableAccessibility: MaterialButton
    private lateinit var btnEnableOverlay: MaterialButton
    private lateinit var btnEnableAdmin: MaterialButton

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        initViews()
        setupListeners()
    }

    override fun onResume() {
        super.onResume()
        updateDashboard()
    }

    private fun initViews() {
        tvStatusTitle = findViewById(R.id.tvStatusTitle)
        tvStatusDesc = findViewById(R.id.tvStatusDesc)
        switchProtection = findViewById(R.id.switchProtection)
        tvScannedCount = findViewById(R.id.tvScannedCount)
        tvBlockedCount = findViewById(R.id.tvBlockedCount)
        tvRulesCount = findViewById(R.id.tvRulesCount)

        btnEnableAccessibility = findViewById(R.id.btnEnableAccessibility)
        btnEnableOverlay = findViewById(R.id.btnEnableOverlay)
        btnEnableAdmin = findViewById(R.id.btnEnableAdmin)

        findViewById<MaterialButton>(R.id.btnManageRules).setOnClickListener {
            startActivity(Intent(this, BlocklistManagerActivity::class.java))
        }

        findViewById<MaterialButton>(R.id.btnSetPin).setOnClickListener {
            showPinDialog()
        }
    }

    private fun setupListeners() {
        switchProtection.setOnCheckedChangeListener { _, isChecked ->
            val stats = NativeBridge.fetchStats()
            if (!isChecked && stats.hasPin) {
                // Require PIN before pausing protection
                promptPinVerification { verified ->
                    if (verified) {
                        NativeBridge.setProtectionActive(false)
                        updateDashboard()
                    } else {
                        switchProtection.isChecked = true
                    }
                }
            } else {
                NativeBridge.setProtectionActive(isChecked)
                updateDashboard()
            }
        }

        btnEnableAccessibility.setOnClickListener {
            val intent = Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
            startActivity(intent)
        }

        btnEnableOverlay.setOnClickListener {
            val intent = Intent(
                Settings.ACTION_MANAGE_OVERLAY_PERMISSION,
                Uri.parse("package:$packageName")
            )
            startActivity(intent)
        }

        btnEnableAdmin.setOnClickListener {
            val component = ComponentName(this, DibaDeviceAdminReceiver::class.java)
            val intent = Intent(DevicePolicyManager.ACTION_ADD_DEVICE_ADMIN).apply {
                putExtra(DevicePolicyManager.EXTRA_DEVICE_ADMIN, component)
                putExtra(
                    DevicePolicyManager.EXTRA_ADD_EXPLANATION,
                    "Enables uninstall protection to prevent disabling Diba."
                )
            }
            startActivity(intent)
        }
    }

    private fun updateDashboard() {
        val stats = NativeBridge.fetchStats()
        switchProtection.isChecked = stats.isActive

        if (stats.isActive) {
            tvStatusTitle.text = getString(R.string.status_protected)
            tvStatusDesc.text = getString(R.string.status_protected_desc)
            tvStatusTitle.setTextColor(getColor(R.color.success))
        } else {
            tvStatusTitle.text = getString(R.string.status_disabled)
            tvStatusDesc.text = getString(R.string.status_disabled_desc)
            tvStatusTitle.setTextColor(getColor(R.color.danger))
        }

        tvScannedCount.text = stats.totalScanned.toString()
        tvBlockedCount.text = stats.totalBlocked.toString()
        tvRulesCount.text = "${stats.rulesCount}+"

        // Update permissions status
        checkPermissions()
    }

    private fun checkPermissions() {
        val dpm = getSystemService(Context.DEVICE_POLICY_SERVICE) as DevicePolicyManager
        val adminComponent = ComponentName(this, DibaDeviceAdminReceiver::class.java)
        val isAdmin = dpm.isAdminActive(adminComponent)

        if (isAdmin) {
            btnEnableAdmin.isEnabled = false
            btnEnableAdmin.text = getString(R.string.status_enabled)
        }

        if (Settings.canDrawOverlays(this)) {
            btnEnableOverlay.isEnabled = false
            btnEnableOverlay.text = getString(R.string.status_enabled)
        }
    }

    private fun showPinDialog() {
        val view = LayoutInflater.from(this).inflate(R.layout.dialog_pin, null)
        val etPin = view.findViewById<EditText>(R.id.etPinInput)

        AlertDialog.Builder(this)
            .setTitle(R.string.pin_setup_title)
            .setView(view)
            .setPositiveButton(R.string.btn_confirm) { _, _ ->
                val pin = etPin.text.toString().trim()
                if (pin.length >= 4) {
                    NativeBridge.setPin(pin)
                    Toast.makeText(this, "PIN configured successfully", Toast.LENGTH_SHORT).show()
                } else {
                    Toast.makeText(this, "PIN must be at least 4 digits", Toast.LENGTH_SHORT).show()
                }
            }
            .setNegativeButton(R.string.btn_cancel, null)
            .show()
    }

    private fun promptPinVerification(callback: (Boolean) -> Unit) {
        val view = LayoutInflater.from(this).inflate(R.layout.dialog_pin, null)
        val etPin = view.findViewById<EditText>(R.id.etPinInput)

        AlertDialog.Builder(this)
            .setTitle(R.string.pin_dialog_title)
            .setMessage(R.string.pin_dialog_desc)
            .setView(view)
            .setPositiveButton(R.string.btn_confirm) { _, _ ->
                val pin = etPin.text.toString().trim()
                if (NativeBridge.verifyPin(pin)) {
                    callback(true)
                } else {
                    Toast.makeText(this, R.string.pin_incorrect, Toast.LENGTH_SHORT).show()
                    callback(false)
                }
            }
            .setNegativeButton(R.string.btn_cancel) { _, _ ->
                callback(false)
            }
            .setCancelable(false)
            .show()
    }
}
