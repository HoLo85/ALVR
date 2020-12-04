#![allow(non_upper_case_globals, non_snake_case, clippy::missing_safety_doc)]

mod logging_backend;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use alvr_common::{data::*, logging::show_err, *};
use jni::{
    objects::*,
    sys::{jobjectArray, jstring},
    *,
};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::slice;

struct OnCreateResultWrapper(OnCreateResult);
unsafe impl Send for OnCreateResultWrapper {}

lazy_static! {
    static ref ON_CREATE_RESULT: Mutex<OnCreateResultWrapper> =
        Mutex::new(OnCreateResultWrapper(<_>::default()));
    static ref REFRESH_RATES: Mutex<Vec<f32>> = Mutex::new(vec![]);
}

#[no_mangle]
pub extern "system" fn Java_com_polygraphene_alvr_OvrActivity_initNativeLogging(
    _: JNIEnv,
    _: JClass,
) {
    logging_backend::init_logging();
}

#[no_mangle]
pub extern "system" fn Java_com_polygraphene_alvr_OvrActivity_createIdentity(
    env: JNIEnv,
    _: JClass,
    jidentity: JObject,
) {
    show_err(|| -> StrResult {
        let identity = create_identity(None)?;

        let jhostname = trace_err!(env.new_string(identity.hostname))?.into();
        trace_err!(env.set_field(jidentity, "hostname", "Ljava/lang/String;", jhostname))?;

        let jcert_pem = trace_err!(env.new_string(identity.certificate_pem))?.into();
        trace_err!(env.set_field(jidentity, "certificatePEM", "Ljava/lang/String;", jcert_pem))?;

        let jkey_pem = trace_err!(env.new_string(identity.key_pem))?.into();
        trace_err!(env.set_field(jidentity, "privateKey", "Ljava/lang/String;", jkey_pem))
    }())
    .ok();
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_LatencyCollector_DecoderInput(
    _: JNIEnv,
    _: JObject,
    frame_index: i64,
) {
    decoderInput(frame_index);
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_LatencyCollector_DecoderOutput(
    _: JNIEnv,
    _: JObject,
    frame_index: i64,
) {
    decoderOutput(frame_index);
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onCreateNative(
    env: JNIEnv,
    activity: JObject,
    asset_manager: JObject,
    jout_result: JObject,
) {
    show_err(|| -> StrResult {
        let result = onCreate(
            env.get_native_interface() as _,
            *activity as _,
            *asset_manager as _,
        );

        trace_err!(env.set_field(
            jout_result,
            "streamSurfaceHandle",
            "I",
            result.streamSurfaceHandle.into()
        ))?;
        trace_err!(env.set_field(
            jout_result,
            "webviewSurfaceHandle",
            "I",
            result.webViewSurfaceHandle.into()
        ))?;
        trace_err!(env.set_field(
            jout_result,
            "loadingSurfaceHandle",
            "I",
            result.loadingSurfaceHandle.into()
        ))?;

        let refresh_rates =
            slice::from_raw_parts(result.refreshRates, result.refreshRatesCount as _).to_vec();
        let default_refresh_rate = refresh_rates.last().cloned().unwrap_or(60_f32);
        trace_err!(env.set_field(
            jout_result,
            "refreshRate",
            "I",
            (default_refresh_rate as i32).into()
        ))?;

        trace_err!(env.set_field(
            jout_result,
            "renderWidth",
            "I",
            (result.recommendedEyeWidth * 2).into()
        ))?;
        trace_err!(env.set_field(
            jout_result,
            "renderHeight",
            "I",
            result.recommendedEyeHeight.into()
        ))?;

        *ON_CREATE_RESULT.lock() = OnCreateResultWrapper(result);
        *REFRESH_RATES.lock() = refresh_rates;

        Ok(())
    }())
    .ok();
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_destroyNative(
    env: JNIEnv,
    _: JObject,
) {
    destroyNative(env.get_native_interface() as _)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_renderNative(
    _: JNIEnv,
    _: JObject,
    rendered_frame_index: i64,
) {
    renderNative(rendered_frame_index)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_renderLoadingNative(
    _: JNIEnv,
    _: JObject,
) {
    renderLoadingNative()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onTrackingNative(
    env: JNIEnv,
    _: JObject,
    udp_receiver_thread: JObject,
) {
    onTrackingNative(env.get_native_interface() as _, *udp_receiver_thread as _)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onResumeNative(
    env: JNIEnv,
    _: JObject,
    jhostname: JString,
    jcertificate_pem: JString,
    jprivate_key: JString,
    jscreen_surface: JObject,
) {
    show_err(|| -> StrResult {
        onResumeNative(env.get_native_interface() as _, *jscreen_surface as _);
        Ok(())
    }())
    .ok();
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onStreamStartNative(
    _: JNIEnv,
    _: JObject,
    width: i32,
    height: i32,
    refresh_rate: i32,
    stream_mic: u8,
    foveation_mode: i32,
    foveation_strength: f32,
    foveation_shape: f32,
    foveation_vertical_offset: f32,
    tracking_space: i32,
) {
    onStreamStartNative(
        width,
        height,
        refresh_rate,
        stream_mic,
        foveation_mode,
        foveation_strength,
        foveation_shape,
        foveation_vertical_offset,
        tracking_space,
    )
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onPauseNative(
    _: JNIEnv,
    _: JObject,
) {
    onPauseNative()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onHapticsFeedbackNative(
    _: JNIEnv,
    _: JObject,
    start_time: i64,
    amplitude: f32,
    duration: f32,
    frequency: f32,
    hand: u8,
) {
    onHapticsFeedbackNative(start_time, amplitude, duration, frequency, hand)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onGuardianSyncAckNative(
    _: JNIEnv,
    _: JObject,
    timestamp: i64,
) {
    onGuardianSyncAckNative(timestamp)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onGuardianSegmentAckNative(
    _: JNIEnv,
    _: JObject,
    timestamp: i64,
    segment_index: i32,
) {
    onGuardianSegmentAckNative(timestamp, segment_index)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_OvrActivity_onBatteryChangedNative(
    _: JNIEnv,
    _: JObject,
    battery: i32,
) {
    onBatteryChangedNative(battery)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_initializeSocket(
    env: JNIEnv,
    instance: JObject,
    hello_port: i32,
    port: i32,
    device_name: JString,
    broadcast_addr_list: jobjectArray,
    refresh_rate: i32,
    render_width: i32,
    render_height: i32,
) {
    initializeSocket(
        env.get_native_interface() as _,
        *instance as _,
        hello_port,
        port,
        **device_name as _,
        broadcast_addr_list as _,
        refresh_rate,
        render_width,
        render_height,
    )
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_closeSocket(
    _: JNIEnv,
    _: JObject,
) {
    closeSocket()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_runLoop(
    env: JNIEnv,
    instance: JObject,
) {
    runLoop(env.get_native_interface() as _, *instance as _)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_interruptNative(
    _: JNIEnv,
    _: JObject,
) {
    interruptNative()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_isConnectedNative(
    _: JNIEnv,
    _: JObject,
) -> u8 {
    isConnectedNative()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_getServerAddress(
    env: JNIEnv,
    _: JObject,
) -> jstring {
    getServerAddress(env.get_native_interface() as _) as _
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_getServerPort(
    _: JNIEnv,
    _: JObject,
) -> i32 {
    getServerPort()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_sendNative(
    _: JNIEnv,
    _: JObject,
    native_buffer: i64,
    buffer_length: i32,
) {
    sendNative(native_buffer, buffer_length)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_polygraphene_alvr_ServerConnection_setSinkPreparedNative(
    _: JNIEnv,
    _: JObject,
    prepared: u8,
) {
    setSinkPreparedNative(prepared)
}
