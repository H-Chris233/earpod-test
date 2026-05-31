// earPod-test: 检测苹果有线耳机线控按钮在 Windows 上触发的虚拟键码。
//
// 使用方法：
//   1. 插上 EarPods（3.5mm / USB-C / Lightning 转接均可）
//   2. cargo run --release
//   3. 按耳机中间按钮，看控制台输出
//   4. Ctrl+C 退出
//
// 如果控制台有 "vkCode=0x00B3" 输出 → 走键盘消息路径，可以做 hold-to-talk。
// 如果没有任何输出 → 走 WM_APPCOMMAND 或 HID Raw Input，需要换方案。

use std::sync::atomic::{AtomicPtr, Ordering};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static HOOK: AtomicPtr<HHOOK> = AtomicPtr::new(std::ptr::null_mut());

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && lparam.0 != 0 {
        let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let msg_type = match wparam.0 as u32 {
            WM_KEYDOWN => "DOWN",
            WM_KEYUP => "UP  ",
            WM_SYSKEYDOWN => "SDOWN",
            WM_SYSKEYUP => "SUP ",
            _ => "OTHER",
        };
        // 只打印非普通字母键，避免刷屏
        if kb.vkCode > 0x5A || msg_type != "UP  " {
            println!(
                "vkCode=0x{:04X} ({:>3}) {}  scanCode=0x{:04X} flags=0x{:04X}",
                kb.vkCode, kb.vkCode, msg_type, kb.scanCode, kb.flags.0
            );
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

fn main() {
    println!("earpod-test — 监听 WH_KEYBOARD_LL...");
    println!("插上 EarPods，按中间按钮，看看有没有输出。");
    println!("按 Ctrl+C 退出。\n");

    unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0);
        let hook = match hook {
            Ok(h) => h,
            Err(e) => {
                eprintln!("SetWindowsHookExW 失败: {e}");
                eprintln!("提示：Windows 低层键盘钩子不需要管理员权限，但防病毒软件可能拦截。");
                return;
            }
        };
        HOOK.store(Box::into_raw(Box::new(hook)), Ordering::SeqCst);
        println!("Hook 已安装，等待按键事件...");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).0 != 0 {
            let _ = TranslateMessage(&msg);
            let _ = windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
        }

        // 清理
        let _ = UnhookWindowsHookEx(hook);
        let _ = Box::from_raw(HOOK.load(Ordering::SeqCst));
    }
}
