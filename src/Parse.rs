trait Map<T> {
    fn map_fields(fields: &Vec<crate::parser::Field>) -> Result<T, VEError>;
}
