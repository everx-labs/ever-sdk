extern crate ton_client;
use self::ton_client::*;

extern crate jni;

use jni::JNIEnv;
use jni::objects::{GlobalRef, JClass, JObject, JString, JValue};

struct JniResultHandler {
    jvm: jni::JavaVM,
    handler: GlobalRef,
}

impl JniResultHandler {
    fn new(env: JNIEnv, handler: JObject) -> JniResultHandler {
        JniResultHandler {
            jvm: env.get_java_vm().unwrap(),
            handler: env.new_global_ref(handler).unwrap(),
        }
    }
}

fn java_value<'a>(env: &JNIEnv<'a>, from: String) -> jni::objects::JValue<'a> {
    JValue::Object(env.new_string(from.as_str()).unwrap().into())
}

fn rust_string(env: &JNIEnv, from: JString) -> String {
    env.get_string(from).unwrap().into()
}

impl JniResultHandler {
    fn on_result(&self, result_json: String, error_json: String, flags: i32) {
        let env = self.jvm.attach_current_thread().unwrap();
        let handler = self.handler.as_obj();
        let java_result_json = java_value(&env, result_json);
        let java_error_json = java_value(&env, error_json);
        let java_flags = JValue::Int(flags);
        env.call_method(
            handler,
            "invoke",
            "(Ljava/lang/String;Ljava/lang/String;I)V",
            &[java_result_json, java_error_json, java_flags],
        ).unwrap();
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern fn Java_ton_sdk_TONSDKJsonApi_request(
    env: JNIEnv,
    _: JClass,
    method: JString,
    params_json: JString,
    on_result: JObject,
) {
    let response = json_sync_request(
        create_context(),//context,
        rust_string(&env, method),
        rust_string(&env, params_json),
    );

    let handler = JniResultHandler::new(env, on_result);

    handler.on_result(response.result_json, response.error_json, 0);
}

