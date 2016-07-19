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
use winapi::minwindef::BOOL;
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
pub unsafe extern "system" fn window_proc(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
	use winapi::winuser::*;

	match msg {
		WM_DESTROY => {
			user32::PostQuitMessage(0);
			return 0 as LRESULT;
		},
		// TODO implement
		WM_MOUSEMOVE => {
			return 0 as LRESULT;
		},
		WM_LBUTTONDOWN => {
			return 0 as LRESULT;
		},
		WM_LBUTTONUP => {
			return 0 as LRESULT;
		},
		_ => {
			return user32::DefWindowProcW(h_wnd, msg, w_param, l_param);
		},
	}
}

// TODO figure out how the heck to use pointers
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
		return false;
	}

	let mut ret = false;
	if user32::OpenClipboard(hwnd) as BOOL == winapi::minwindef::TRUE {
		if user32::EmptyClipboard() as BOOL == winapi::minwindef::TRUE {
			if user32::SetClipboardData(CF_BITMAP, shot_bitmap) as BOOL == winapi::minwindef::TRUE {
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
			m_class_name,
			m_class_name,
			style,
			// dimensions
			0, 0, rect.x2, rect.y2,
			0 as HBRUSH, // no parent
			0 as HWND, // no menu
			hInstance, // module_instance
			0 as HMENU,
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