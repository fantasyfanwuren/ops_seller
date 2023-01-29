use ops_seller::*;
#[tokio::main]
async fn main() {
    // 时间检测
    println!("******************************************************");
    println!("鸣谢大海(WX:wxid_0l4pnrb4i0d512),对本开源软件的捐赠!");
    println!("******************************************************");
    let mut seller = Seller::new().await;
    seller.wait_for_password();
    println!("开始执行");
    seller.run_sell().await;
}
