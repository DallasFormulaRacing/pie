
struct Device<Status> {
    status: Status,
    name: String,
    system: System,
}
struct Online;
struct Offline;

impl Device<Status> {
    fn ping(&self) -> Result<(Device<Online>)> {

    }
}

impl Device<Online> {

}
impl Device<Offline> {
}
