extern crate kernel32;
extern crate user32;
extern crate winapi;
extern crate libc;
extern crate gdi32;

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
use winapi::winuser::CF_BITMAP;

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

const g_class_name: &'static str = "winshot";

// Rectangle represents screen coordinates
struct Rectangle {
	x: i32,
	y: i32,
	x2: i32,
	y2: i32,
} 

// convert normal string to wide string
#[allow(dead_code)]
fn to_wstring(str: &str) -> *const u16 {
	let v: Vec<u16> = OsStr::new(str).encode_wide() . chain(Some(0).into_iter()).collect();
	v.as_ptr()
}

// This is our window message handler function. Currently, 
// it only processes the WM_DESTROY message to exit our window properly on close event.
#[allow(dead_code)]
pub unsafe extern "system" fn window_proc(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	if msg == winapi::winuser::WM_DESTROY {
		user32::PostQuitMessage(0);
	} else if msg == winapi::winuser::WM_MOUSEMOVE {
		return 0 as LRESULT;
	} else if msg == winapi::winuser::WM_LBUTTONDOWN {
		return 0 as LRESULT;
	} else if msg == winapi::winuser::WM_LBUTTONUP {
		return 0 as LRESULT;
	} else {
		return user32::DefWindowProcW(h_wnd, msg, w_param, l_param);
	}
}

// The following function hides the Console Window since we are building a GUI application.
#[allow(dead_code)]
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

fn normalize_coords(rect: Rectangle) -> Rectangle {
    let Rectangle{mut x, mut y, mut x2, mut y2} = rect;

    if x > x2 {
        let tmp = x;
        x = x2;
        x2 = tmp;
    }
    if y > y2 {
        let tmp = y;
        y = y2;
        y2 = tmp;
    }
    
    Rectangle{x: x, y: y, x2: x2, y2: y2}
}

fn get_screen_rect() -> Rectangle {
	Rectangle{
		x /* left */: user32::GetSystemMetrics(winapi::winuser::SM_XVIRTUALSCREEN),
		y /* top */: user32::GetSystemMetrics(winapi::winuser::SM_YVIRTUALSCREEN),
		x2 /* right */: user32::GetSystemMetrics(winapi::winuser::SM_CXVIRTUALSCREEN) + user32::GetSystemMetrics(winapi::winuser::SM_XVIRTUALSCREEN),
		y2 /* bottom */: user32::GetSystemMetrics(winapi::winuser::SM_CYVIRTUALSCREEN) + user32::GetSystemMetrics(winapi::winuser::SM_CYVIRTUALSCREEN),
	}
}

fn capture_screen_clipboard(hwnd: HWND, mut rect: Rectangle) -> bool {
	// normalize coordinates
	let rect = normalize_coords(rect);
	let w = rect.x2 - rect.x;
	let h = rect.y2 - rect.y;

	let screen_dc = user32::GetDC(0 as HWND);
	let shot_dc = gdi32::CreateCompatibleDC(screen_dc);
	let shot_bitmap = gdi32::CreateCompatibleBitmap(screen_dc, w, h);
	let old_obj = gdi32::SelectObject(shot_dc, shot_bitmap);
	
	if !gdi32::BitBlt(shot_dc, 0, 0, w, h, screen_dc, rect.x, rect.y, 0 as DWORD) {
		println!("{:?}", "BitBlt failed? Perhaps.");
		false
	}

	let mut ret = false;
	if user32::OpenClipboard(hwnd) {
		if user32::EmptyClipboard() {
			if user32::SetClipboardData(CF_BITMAP, shot_bitmap) {
				ret = true;
			}
			user32::CloseClipboard();
		}
	} else {
		println!("{:?}", "could not open clipboard");
	}

	// Clean up
	gdi32::DeleteDC(shot_dc);
	gdi32::DeleteDC(screen_dc);
	gdi32::SelectObject(shot_dc, old_obj);

	return ret;
}

fn main() {
	unsafe {
		let hInstance = kernel32::GetModuleHandleW(0 as LPCWSTR);
		let m_class_name = to_wstring(g_class_name);
		let wnd = WNDCLASSW {
			hInstance: hInstance,
			lpszClassName: m_class_name,
			lpszMenuName: 0 as LPCWSTR,
			lpfnWndProc: Some(window_proc),
			cbClsExtra: 0,
			cbWndExtra: 0,
			style: winapi::winuser::CS_HREDRAW | winapi::winuser::CS_VREDRAW,
			hbrBackground: 0,
			hCursor: user32::LoadCursorW(0 as HINSTANCE, winapi::winuser::IDC_CROSS),
			hIcon: user32::LoadIconW(0 as HINSTANCE, winapi::winuser::IDI_APPLICATION),
		};

		user32::RegisterClassW(&wnd);
		let rect = get_screen_rect();
		let exstyle = winapi::winuser::WS_EX_TRANSPARENT;
		let style = winapi::winuser::WS_POPUP;

		let win = user32::CreateWindowExW(exstyle,
			m_class_name.as_ptr() as *mut _,
			m_class_name.as_ptr() as *mut _,
			style,
			// dimensions
			0, 0, rect.x2, rect.y2,
			0, // no parent
			0 as HWND, // no menu
			hInstance, // module_instance
			0 as LPARAM,
		);

		user32::ShowWindow(win, winapi::winuser::SW_SHOW);
		user32::ShowWindow(win, winapi::winuser::SW_SHOWMAXIMIZED);
		user32::SetForegroundWindow(win);
		
		capture_screen_clipboard(win, Rectangle{
			x: 0,
			y: 0,
			x2: 200,
			y2: 200,
		});
		user32::DestroyWindow(win);
	}
}

// fn main() {
// 	// Here our unsafe code goes
// 	unsafe {
// 		// First we hide the console window
// 		hide_console_window();

// 		// Then we initialize the WNDCLASS structure
// 		let m_class_name = to_wstring(g_class_name);
// 		let wnd = WNDCLASSW {
// 			hInstance: 0 as HINSTANCE,
// 			lpszClassName = m_class_name,
// 			lpszMenuName: 0 as LPCWSTR,
// 			lpfnWndProc: Some(window_proc),

// 			style: winapi::winuser::CS_HREDRAW | winapi::winuser::CS_VREDRAW,
			
// 			cbClsExtra: 0,
// 			cbWndExtra: 0,
			
// 			hIcon: user32::LoadIconW(0 as HINSTANCE, winapi::winuser::IDI_APPLICATION),
// 			hCursor: user32::LoadCursorW(0 as HINSTANCE, winapi::winuser::IDI_APPLICATION),
// 			hbrBackground: 16 as HBRUSH,
			
			
// 		};

// 		// Register our class
// 		user32::RegisterClassW(&wnd);

// 		let h_wnd_desktop = user32::GetDesktopWindow();

// 		user32::CreateWindowExA(0, "my_window".as_ptr() as *mut _,
// 			"Simple Window".as_ptr() as *mut _, WS_OVERLAPPEDWINDOW | WS_VISIBLE,
// 			0, 0, 400, 400, h_wnd_desktop, 0 as HMENU, 0 as HINSTANCE,
// 			std::ptr::null_mut());

// 		let mut msg = winapi::winuser::MSG {
// 			hwnd: 0 as HWND,
// 			message: 0 as UINT,
// 			wParam: 0 as WPARAM,
// 			lParam: 0 as LPARAM,
// 			time: 0 as DWORD,
// 			pt : winapi::windef::POINT{ x: 0, y: 0, },
// 		};

// 		// Kick off the application loop
// 		loop {
// 			let pm = user32::PeekMessageW(&mut msg, 0 as HWND, 0, 0, winapi::winuser::PM_REMOVE);

// 			if pm == 0 {
// 				continue;
// 			}

// 			if msg.message == winapi::winuser::WM_QUIT {
// 				break;
// 			}

// 			user32::TranslateMessage(&mut msg);
// 			user32::DispatchMessageW(&mut msg);
// 		}
// 	}
// }