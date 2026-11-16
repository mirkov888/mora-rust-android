package com.mora.rustandroid
object NativeCore {
 init { System.loadLibrary("mora_rust_android") }
 @JvmStatic external fun nativeCoreApiVersion(): Int
 @JvmStatic external fun nativeCoreCapabilities(): Int
 @JvmStatic external fun nativeAdd(left:Int,right:Int):Int
 @JvmStatic external fun nativeNormalizeProfileLabel(input:String):String?
 @JvmStatic external fun nativeIsValidProfileId(input:String):Boolean
 @JvmStatic external fun nativeIsSafeText(input:String,maxBytes:Int):Boolean
 @JvmStatic external fun nativeProfileOperation(storeJson:String,operation:String,id:String,label:String,timestamp:Long):String
 @JvmStatic external fun nativeSystemConfigOperation(configJson:String,operation:String,key:String,value:String,timestamp:Long):String
}
