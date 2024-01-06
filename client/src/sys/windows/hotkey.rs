use windows::Win32::{
    Foundation::HWND,
    UI::Input::KeyboardAndMouse::{MOD_CONTROL, VK_U},
};

pub fn create_listener<F>(callback: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Result<(), Box<dyn std::error::Error>>,
{
    unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(
            HWND::default(),
            1,
            MOD_CONTROL,
            VK_U.0 as u32,
        )?;

        loop {
            // This okay as the value is never read until its initialised
            #[allow(invalid_value)]
            let mut msg = std::mem::MaybeUninit::uninit().assume_init();
            while windows::Win32::UI::WindowsAndMessaging::GetMessageW(
                &mut msg,
                HWND::default(),
                0,
                0,
            )
            .as_bool()
            {
                // Why are we doing this?
                if msg.wParam.0 == 0 {
                    continue;
                }

                callback()?;
            }
        }
    }
}
