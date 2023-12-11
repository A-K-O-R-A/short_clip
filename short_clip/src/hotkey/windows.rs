pub fn create_listener<F>(callback: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn() -> Result<(), Box<dyn std::error::Error>>,
{
    callback()
}
