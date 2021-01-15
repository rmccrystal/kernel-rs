/// Converts a Windows error code to its corresponding message.
/// If there is no message associated with the code, this will return None
pub(crate) fn error_code_to_message(code: u32) -> Option<String> {
    let mut message_buf: [i8; 512] = [0; 512];

    // Get the error string by the code
    let buf_len = unsafe {
        FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            NULL,
            code,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
            message_buf.as_mut_ptr(),
            512,
            NULL as *mut *mut i8,
        )
    };

    // there is no message for the error
    if buf_len == 0 {
        return None;
    }

    let mut error_string = c_char_array_to_string(message_buf.to_vec());

    // Remove \n from end of string
    error_string.pop();
    // Remove \r from end of string
    error_string.pop();

    Some(error_string)
}
