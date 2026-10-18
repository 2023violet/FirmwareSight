/* Written for this project; no third-party code copied. */
static const short imu_calibration[4] = { 0, 37, 74, 10 };
static int imu_history[4];

int imu_read_with_a_name_long_enough_to_make_a_linker_wrap_its_input_section_row(int axis)
{
    int value = imu_calibration[axis % 4] * 3;
    imu_history[axis % 4] = value;
    return value;
}

int imu_read(int axis)
{
    return imu_read_with_a_name_long_enough_to_make_a_linker_wrap_its_input_section_row(axis);
}
