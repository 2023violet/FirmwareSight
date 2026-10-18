/* Written for this project; no third-party code copied. */
static const short imu_calibration[24] = { 0, 37, 74, 10, 47, 84, 20, 57, 94, 30, 67, 3, 40, 77, 13, 50, 87, 23, 60, 97, 33, 70, 6, 43 };
static int imu_history[24];

int imu_read_with_a_name_long_enough_to_make_a_linker_wrap_its_input_section_row(int axis)
{
    int value = imu_calibration[axis % 24] * 3;
    imu_history[axis % 24] = value;
    return value;
}

int imu_read(int axis)
{
    return imu_read_with_a_name_long_enough_to_make_a_linker_wrap_its_input_section_row(axis);
}
