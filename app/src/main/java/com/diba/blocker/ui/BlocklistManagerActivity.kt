package com.diba.blocker.ui

import android.os.Bundle
import android.widget.ArrayAdapter
import android.widget.EditText
import android.widget.ListView
import android.widget.RadioGroup
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import com.diba.blocker.R
import com.diba.blocker.core.NativeBridge
import com.google.android.material.button.MaterialButton

class BlocklistManagerActivity : AppCompatActivity() {

    private val customRules = mutableListOf<String>()
    private lateinit var adapter: ArrayAdapter<String>

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_blocklist_manager)

        val etInput = findViewById<EditText>(R.id.etRuleInput)
        val rgType = findViewById<RadioGroup>(R.id.rgRuleType)
        val btnAdd = findViewById<MaterialButton>(R.id.btnAddRule)
        val lvRules = findViewById<ListView>(R.id.lvRules)

        adapter = ArrayAdapter(this, android.R.layout.simple_list_item_1, customRules)
        lvRules.adapter = adapter

        btnAdd.setOnClickListener {
            val text = etInput.text.toString().trim()
            if (text.isEmpty()) {
                Toast.makeText(this, "Please enter a domain or keyword", Toast.LENGTH_SHORT).show()
                return@setOnClickListener
            }

            when (rgType.checkedRadioButtonId) {
                R.id.rbBlacklist -> {
                    NativeBridge.addBlockedDomain(text)
                    customRules.add("[BLOCKED DOMAIN] $text")
                    Toast.makeText(this, "Domain added to blocklist", Toast.LENGTH_SHORT).show()
                }
                R.id.rbWhitelist -> {
                    NativeBridge.addWhitelistedDomain(text)
                    customRules.add("[WHITELIST] $text")
                    Toast.makeText(this, "Domain added to whitelist", Toast.LENGTH_SHORT).show()
                }
                R.id.rbKeyword -> {
                    NativeBridge.addBlockedKeyword(text)
                    customRules.add("[KEYWORD] $text")
                    Toast.makeText(this, "Keyword added to filter", Toast.LENGTH_SHORT).show()
                }
            }

            adapter.notifyDataSetChanged()
            etInput.text.clear()
        }
    }
}
