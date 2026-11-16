package com.mora.rustandroid
import android.app.Activity
import android.graphics.Typeface
import android.os.Bundle
import android.view.Gravity
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import org.json.JSONObject

class MainActivity : Activity() {
 private lateinit var output:TextView
 private lateinit var idInput:EditText
 private lateinit var labelInput:EditText
 private val dataFile by lazy { java.io.File(filesDir,"profiles.json") }
 override fun onCreate(savedInstanceState:Bundle?){
  super.onCreate(savedInstanceState)
  val column=LinearLayout(this).apply{orientation=LinearLayout.VERTICAL;setPadding(24,28,24,28)}
  fun label(s:String,size:Float=15f)=TextView(this).apply{text=s;textSize=size;gravity=Gravity.CENTER}
  column.addView(label("Mora Rust Android Core",24f).apply{typeface=Typeface.DEFAULT_BOLD})
  column.addView(label("Profile Manager · JNI API v3"))
  idInput=EditText(this).apply{hint="Profile ID, contoh: daily_01"}
  labelInput=EditText(this).apply{hint="Nama profil"}
  column.addView(idInput);column.addView(labelInput)
  column.addView(button("Buat Profil"){"create"})
  column.addView(button("Ubah Profil"){"update"})
  column.addView(button("Hapus Profil"){"delete"})
  column.addView(button("Muat / Lihat Profil"){"list"})
  column.addView(Button(this).apply{text="Jalankan Semua Rust Tests";setOnClickListener{runTests()}})
  output=label("Profile store siap.")
  column.addView(output)
  setContentView(ScrollView(this).apply{addView(column)})
  runProfile("list")
 }
 private fun button(title:String,op:()->String)=Button(this).apply{text=title;setOnClickListener{runProfile(op())}}
 private fun storeJson()=if(dataFile.exists())dataFile.readText(Charsets.UTF_8) else """{"profiles":[]}"""
 private fun runProfile(op:String){
  try{
   val response=JSONObject(NativeCore.nativeProfileOperation(storeJson(),op,idInput.text.toString().trim(),labelInput.text.toString(),System.currentTimeMillis()))
   if(!response.optBoolean("ok")){output.text="Gagal: "+response.optString("error");return}
   val json=response.getJSONObject("store").toString(2)
   dataFile.writeText(json,Charsets.UTF_8)
   output.text="Berhasil: "+op+"\n"+json
  }catch(e:Exception){output.text="Error: "+(e.localizedMessage?:e.javaClass.simpleName)}
 }
 private fun runTests(){
  try{
   val checks=mutableListOf(
    NativeCore.nativeCoreApiVersion()==3,NativeCore.nativeCoreCapabilities()==31,
    NativeCore.nativeAdd(20,22)==42,NativeCore.nativeNormalizeProfileLabel(" Daily ")=="Daily",
    NativeCore.nativeNormalizeProfileLabel("  ")==null,NativeCore.nativeIsValidProfileId("daily_01"),
    !NativeCore.nativeIsValidProfileId("../bad"),NativeCore.nativeIsSafeText("hello",16),
    !NativeCore.nativeIsSafeText("bad\ntext",16))
   val created=JSONObject(NativeCore.nativeProfileOperation("""{"profiles":[]}""","create","test_01"," Test Profile ",10))
   checks.add(created.optBoolean("ok")&&created.getJSONObject("store").getJSONArray("profiles").length()==1)
   val updated=JSONObject(NativeCore.nativeProfileOperation(created.getJSONObject("store").toString(),"update","test_01","Updated",20))
   checks.add(updated.optBoolean("ok")&&updated.getJSONObject("store").getJSONArray("profiles").getJSONObject(0).getString("label")=="Updated")
   val deleted=JSONObject(NativeCore.nativeProfileOperation(updated.getJSONObject("store").toString(),"delete","test_01","",30))
   checks.add(deleted.optBoolean("ok")&&deleted.getJSONObject("store").getJSONArray("profiles").length()==0)
   val pass=checks.count{it}
   output.text=(if(pass==checks.size)"SEMUA TES BERHASIL" else "ADA TES GAGAL")+"\nTes: "+pass+"/"+checks.size+"\nAPI v3 · Capability 0x1f"
  }catch(e:Exception){output.text="Tes gagal: "+(e.localizedMessage?:e.javaClass.simpleName)}
 }
}
