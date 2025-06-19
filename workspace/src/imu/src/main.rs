mod args;

use std::env;

use rclrs;

use sensor_msgs::msg::Imu;
use sensor_msgs::msg::Temperature;

use driver;
use driver::protocol::adis;

fn main() {
    let context = rclrs::Context::new(env::args())
        .expect("ROS2 ADIS IMU: Could not initialize context.");

    let node = rclrs::create_node(&context, "adis_imu_node")
        .expect("ROS2 ADIS IMU: Could not create node.");

    let publisher_imu = node
        .create_publisher::<Imu>(args::TOPIC_NAME_IMU, rclrs::QOS_PROFILE_DEFAULT)
        .expect("ROS2 ADIS IMU: Could not create publisher for imu.");

    let publisher_temp = node
        .create_publisher::<Temperature>(args::TOPIC_NAME_TEMP, rclrs::QOS_PROFILE_DEFAULT)
        .expect("ROS2 ADIS IMU: Could not create publisher for temperature.");

    let mut adis = driver::AdisDevice::from_vid_pid(args::VID, args::PID, args::BAUD_RATE, args::VERSION, None)
        .expect("ROS2 ADIS IMU: Could not open device.");

    adis.send_restart().expect("ROS2 ADIS IMU: Could not restart device.");

    adis.send_config(driver::protocol::cfg::CFG::Burst32(args::CGF_BURST_MODE))
        .expect("ROS2 ADIS IMU: Could not set burst mode.");

    adis.send_config(driver::protocol::cfg::CFG::BurstSel(args::CGF_BURST_SEL))
        .expect("ROS2 ADIS IMU: Could not set burst sel.");

    adis.send_config(driver::protocol::cfg::CFG::BurstEn(true))
        .expect("ROS2 ADIS IMU: Could not enable burst.");

    while context.ok() {
        let messages = adis.expect_burst().expect("ROS2 ADIS IMU: There was error while reading.");

        for m in messages {
            if m.corrupted {
                continue;
            }

            let mut imu_message = Imu::default();
            let mut temp_message = Temperature::default();

            match m.data {
                adis::Sel::Sel0 {
                    x_gyro,
                    y_gyro,
                    z_gyro,
                    x_accl,
                    y_accl,
                    z_accl,
                } => {
                    imu_message.angular_velocity.x = x_gyro.get::<adis::radian_per_second>();
                    imu_message.angular_velocity.y = y_gyro.get::<adis::radian_per_second>();
                    imu_message.angular_velocity.z = z_gyro.get::<adis::radian_per_second>();

                    imu_message.linear_acceleration.x = x_accl.get::<adis::meter_per_second_squared>();
                    imu_message.linear_acceleration.y = y_accl.get::<adis::meter_per_second_squared>();
                    imu_message.linear_acceleration.z = z_accl.get::<adis::meter_per_second_squared>();
                }
                _ => {}
            }

            temp_message.temperature = m.temp.get::<adis::degree_celsius>();

            publisher_imu
                .publish(imu_message)
                .expect("ROS2 ADIS IMU: Could not publish imu message.");

            publisher_temp
                .publish(temp_message)
                .expect("ROS2 ADIS IMU: Could not publish temp message.");
        }
    }

    adis.send_restart().ok();
}
