extern crate kernel32;
extern crate user32;
extern crate winapi;
extern crate libc;

use winapi::windef::HWND;
use winapi::windef::HMENU;
use winapi::windef::HBRUSH;
use winapi::minwindef::HINSTANCE;

use winapi::minwindef::UINT;
use winapi::minwindef::DWORD;
use winapi::minwindef::WPARAM;
use winapi::minwindef::LPARAM;
use winapi::minwindef::LRESULT;
use winapi::winnt::LPCWSTR;

use winapi::winuser::WS_OVERLAPPEDWINDOW;
use winapi::winuser::WS_VISIBLE;
use winapi::winuser::WNDCLASSW;

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

// convert normal string to wide string
fn to_wstring(str: &str) -> *const u16 {
	let v: Vec<u16> = OsStr::new(str).encode_wide() . chain(Some(0).into_iter()).collect();
	v.as_ptr()
}

// This is our window message handler function. Currently, 
// it only processes the WM_DESTROY message to exit our window properly on close event.
pub unsafe extern "system" fn window_proc(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	if msg == winapi::winuser::WM_DESTROY {
		user32::PostQuitMessage(0);
	}
	user32::DefWindowProcW(h_wnd, msg, w_param, l_param)
}

// The following function hides the Console Window since we are building a GUI application.
fn hide_console_window() {
	let window = unsafe {
		kernel32::GetConsoleWindow()
	};

	if window != std::ptr::null_mut() {
		unsafe {
			user32::ShowWindow(window, winapi::SW_HIDE)
		};
	}
}

fn main() {
	// Here our unsafe code goes
	unsafe {
		// First we hide the console window
		hide_console_window();

		// Then we initialize the WNDCLASS structure
		let class_name = to_wstring("my_window");
		let wnd = WNDCLASSW {
			style: 0,
			lpfnWndProc: Some(window_proc),
			cbClsExtra: 0,
			cbWndExtra: 0,
			hInstance: 0 as HINSTANCE,
			hIcon: user32::LoadIconW(0 as HINSTANCE, winapi::winuser::IDI_APPLICATION),
			hCursor: user32::LoadCursorW(0 as HINSTANCE, winapi::winuser::IDI_APPLICATION),
			hbrBackground: 16 as HBRUSH,
			lpszMenuName: 0 as LPCWSTR,
			lpszClassName: class_name,
		};

		// Register our class
		user32::RegisterClassW(&wnd);

		let h_wnd_desktop = user32::GetDesktopWindow();

		user32::CreateWindowExA(0, "my_window".as_ptr() as *mut _,
			"Simple Window".as_ptr() as *mut _, WS_OVERLAPPEDWINDOW | WS_VISIBLE,
			0, 0, 400, 400, h_wnd_desktop, 0 as HMENU, 0 as HINSTANCE,
			std::ptr::null_mut());

		let mut msg = winapi::winuser::MSG {
			hwnd: 0 as HWND,
			message: 0 as UINT,
			wParam: 0 as WPARAM,
			lParam: 0 as LPARAM,
			time: 0 as DWORD,
			pt : winapi::windef::POINT { x: 0, y: 0, },
		};

		// Kick off the application loop
		loop {
			let pm = user32::PeekMessageW(&mut msg, 0 as HWND, 0, 0, winapi::winuser::PM_REMOTE);

			if pm == 0 {
				continue;
			}

			if msg.message == winapi::winuser::WM_QUIT {
				break;
			}

			user32::TranslateMessage(&mut msg);
			user32::DispatchMessageW(&mut msg);
		}
	}
}