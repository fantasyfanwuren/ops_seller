use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufWriter;
use std::{
    fs::{self, File},
    io,
    path::Path,
};
use thirtyfour::prelude::*;
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Debug)]
struct Selled {
    items: Vec<usize>,
}

pub struct Seller {
    driver: WebDriver,
    price: f64,
    address: String,
    start: usize,
    end: usize,
    selled: Selled,
}

impl Seller {
    pub async fn new() -> Self {
        // 通知用户设置 address: String,合约地址
        let mut address = String::new();
        println!("请输入您的NFT集合根地址\n
        (例如:https://opensea.io/zh-CN/assets/matic/0x27e4c854fd05e672e599bad2dca9aef1c8961b99/):\n");
        io::stdin()
            .read_line(&mut address)
            .expect("读取合约地址失败");
        let address = address.trim().to_string();

        // 通知用户设置NFT起始编号 start
        let mut start = String::new();
        println!("请输入设置NFT售价的起始编号:(例如:123)");
        io::stdin()
            .read_line(&mut start)
            .expect("读取设置NFT售价的起始编号失败");
        let start: usize = start
            .trim()
            .parse()
            .expect("无法将输入的内容解析为起始编号信息");

        // 通知用户设置NFT结束变化end
        let mut end = String::new();
        println!("请输入您设置NFT售价的结束编号:(例如:20222)");
        io::stdin()
            .read_line(&mut end)
            .expect("读取设置NFT售价的结束编号失败");
        let end: usize = end.trim().parse().expect("无法将输入的内容解析为结尾编号");

        // 通知用户设置价格
        let mut price = String::new();
        println!("请输入您要设置的价格:");
        io::stdin().read_line(&mut price).expect("读取价格失败!");
        let price: f64 = price
            .trim()
            .parse()
            .expect("无法将您输入的内容解析为价格数据");

        // 记忆文件是否采用设置
        let mut selled = Selled { items: vec![] };
        if Path::new("selled.json").exists() {
            println!("检测到有之前的设置售价记录,是否采用该记录(是:y/否:n)");
            let mut use_selled = String::new();
            io::stdin()
                .read_line(&mut use_selled)
                .expect("读取是否采用设置售价记录出错");
            if use_selled.trim() == "y" {
                let selled_string =
                    fs::read_to_string("selled.json").expect("读取设置售价记录文件失败");
                selled = serde_json::from_str(&selled_string)
                    .expect("将售价记录文本转化为Sellde结构失败");
            }
        }

        // 询问用户端口号port
        let mut port = String::new();
        println!("请输入一个端口号,建议:9500-9600之间,多开请设置不同的端口号:");
        io::stdin().read_line(&mut port).expect("读取端口号失败");
        let port: usize = port.trim().parse().expect("解析端口号失败");
        let port_arg = format!("--port={}", port);
        // 打开chromdriver
        std::process::Command::new("./chromedriver.exe")
            .arg("--ip=localhost")
            .arg(&port_arg)
            .spawn()
            .expect("无法打开chromedriver.exe");
        // 创建driver
        let server_url = format!("http://localhost:{}", port);
        let caps = DesiredCapabilities::chrome();
        let driver = match WebDriver::new(&server_url, caps).await {
            Ok(d) => d,
            Err(_) => panic!("创建chromdriver失败,可能因为无法启动ChormDriver.exe"),
        };

        // 让网页转到opensea
        driver
            .goto("https://opensea.io/")
            .await
            .expect("打开opensea网站失败,请检查您是否启动了ChromDriver,或者检查您的网络");

        Seller {
            driver,
            price,
            selled,
            address,
            start,
            end,
        }
    }

    pub async fn goto(&self, url: &str) {
        self.driver.goto(url).await.expect("前往{url}失败");
        // let delay = Duration::new(60, 0);
        // self.driver
        //     .set_page_load_timeout(delay)
        //     .await
        //     .expect("设置页面加载时间失败");
    }

    pub fn wait_for_password(&self) {
        let mut is_continue = String::new();
        println!("请您在网页中添加钱包信息,并手动转到profile目录下,然后回到本页面按下:y");
        io::stdin()
            .read_line(&mut is_continue)
            .expect("读取价格失败!");
        if is_continue.trim() != "y" {
            panic!("按照您的要求,即将退出程序");
        }
    }

    pub async fn run_sell(&mut self) {
        for id in self.start..=self.end {
            self.set_price(id).await;
            // 保存设置售价记录
            let selled_string = match serde_json::to_string(&self.selled) {
                Ok(s) => s,
                Err(_) => {
                    log("将售价设置记录转化为json文本失败");
                    panic!("将售价设置记录转化为json文本失败");
                }
            };
            if !Path::new("selled.json").exists() {
                File::create("selled.json").expect("创建售价设置记录文件失败");
            }

            if let Err(_) = fs::write("selled.json", &selled_string) {
                log("售价设置记录写入失败");
                panic!("售价设置记录写入失败");
            }
        }
        drop(self);
        // self.driver.quit().await.expect("msg");
    }

    pub async fn set_price(&mut self, id: usize) {
        let wait_time = 10;
        if self.selled.items.contains(&id) {
            println!("{}已经设置过,跳过", id);
            return;
        }
        let url = format!("{}{}", self.address, id);
        // 页面转到id所在的地址内
        self.goto(&url).await;

        // 无限等待页面找到售卖按钮
        let sell_button = loop {
            match self
                .driver
                .query(By::Css(
                    "button[class='sc-29427738-0 sc-788bb508-0 brbNiF bBXuZv']",
                ))
                .or(By::Css(
                    "a[class='sc-1f719d57-0 fKAlPV sc-29427738-0 sc-788bb508-0 brbNiF bBXuZv']",
                ))
                .wait(Duration::from_secs(wait_time), Duration::from_secs(1))
                .first()
                .await
            {
                Ok(s) => break s,
                Err(_) => {
                    println!("获取售卖按钮失败");
                    log("获取售卖按钮失败");
                    sleep(Duration::from_secs(2)).await;
                }
            }
        };

        match sell_button
            .attr("class")
            .await
            .expect("获取售价按钮class失败")
        {
            None => return,
            Some(name) => {
                if name == "sc-29427738-0 sc-788bb508-0 brbNiF bBXuZv" {
                    println!("{}之前已经完成挂单,添加到售价设置记录中", id);
                    self.selled.items.push(id);
                    return;
                }
                if name == "sc-1f719d57-0 fKAlPV sc-29427738-0 sc-788bb508-0 brbNiF bBXuZv" {
                    // 未卖出
                    if let Err(_) = sell_button.click().await {
                        log("点击售卖按钮失败");
                        println!("点击售卖按钮失败");
                    }
                }
            }
        }
        // 填写价格并提交
        // <input aria-invalid="false" id="price" name="price" placeholder="金额" value="" style="cursor: text;">
        let input_price = loop {
            match self
                .driver
                .query(By::Id("price"))
                .wait(Duration::from_secs(wait_time), Duration::from_secs(1))
                .first()
                .await
            {
                Ok(s) => break s,
                Err(_) => {
                    println!("获取input_price失败");
                    log("获取input_price失败");
                    sleep(Duration::from_secs(2)).await;
                }
            }
        };

        if let Err(_) = input_price.send_keys(self.price.to_string()).await {
            log("提交价格表单失败");
            panic!("提交价格表单失败");
        }

        println!("{}设置价格为:{}", id, self.price.to_string());

        // 点击提交按键
        // <button type="submit" width="100%" class="sc-29427738-0 sc-788bb508-0 kqzAEQ bBXuZv">Complete listing</button>
        let submit_price = loop {
            match self
                .driver
                .query(By::Css("button[type='submit']"))
                .wait(Duration::from_secs(wait_time), Duration::from_secs(1))
                .first()
                .await
            {
                Ok(s) => break s,
                Err(_) => {
                    println!("检索提交按钮失败");
                    log("检索提交按钮失败");
                    sleep(Duration::from_secs(2)).await;
                }
            }
        };

        // 切换窗口
        let mut windows = loop {
            match self.driver.windows().await {
                Ok(s) => break s,
                Err(_) => {
                    log("获取窗口数量失败");
                    println!("获取窗口数量失败");
                    sleep(Duration::from_secs(1)).await;
                }
            };
        };

        let org_len = windows.len();
        if let Err(_) = submit_price.click().await {
            log("点击提交按钮失败");
            panic!("点击提交按钮失败");
        }

        // 切换窗口4
        loop {
            windows = loop {
                match self.driver.windows().await {
                    Ok(s) => break s,
                    Err(_) => {
                        log("等待签名窗口出现失败,继续等待..");
                        println!("等待签名窗口出现失败");
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            };

            if windows.len() != org_len {
                if let Err(_) = self
                    .driver
                    .switch_to_window(windows[windows.len() - 1].clone())
                    .await
                {
                    log("切换签名窗口失败");
                    println!("切换签名窗口失败");
                }
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }

        // 点击滚轮
        let arrow_down = loop {
            match self
                .driver
                .query(By::Css("i[class='fa fa-arrow-down']"))
                .wait(Duration::from_secs(wait_time), Duration::from_secs(1))
                .first()
                .await
            {
                Ok(s) => break s,
                Err(_) => {
                    log("获取签名窗口滚轮按键失败");
                    println!("获取签名窗口滚轮按键失败");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        };

        if let Err(_) = arrow_down.click().await {
            log("点击滚轮按键失败");
            panic!("点击滚轮按键失败");
        }

        // 点击签名
        // <button class="button btn--rounded btn-primary" data-testid="signature-sign-button" role="button" tabindex="0">签名</button>
        let sign = loop {
            match self
                .driver
                .query(By::Css("button[data-testid='signature-sign-button']"))
                .wait(Duration::from_secs(wait_time), Duration::from_secs(1))
                .first()
                .await
            {
                Ok(s) => break s,
                Err(_) => {
                    log("获取签名按钮失败");
                    println!("获取签名按钮失败");
                    sleep(Duration::from_secs(2)).await;
                }
            }
        };
        if let Err(_) = sign.click().await {
            log("点击签名按钮失败");
            panic!("点击签名按钮失败");
        }
        println!("进行签名操作..");

        // 结束后返回
        loop {
            sleep(Duration::from_secs(1)).await;
            windows = match self.driver.windows().await {
                Ok(s) => s,
                Err(_) => {
                    log("等待签名窗口消失失败");
                    panic!("等待签名窗口消失失败");
                }
            };

            if windows.len() == org_len {
                break;
            }
        }
        if let Err(_) = self.driver.switch_to_window(windows[0].clone()).await {
            log("切换回原始窗口失败");
            panic!("切换回原始窗口失败");
        }

        self.selled.items.push(id);
        println!("{}设置售价完成", id);
        log(&format!("{}设置售价完成", id));
    }
}

pub fn log(contents: &str) {
    let file_path = "log.txt";
    // 判断文件是否存在
    // OpenOptions::new().create(true).append(true).open("/tmp/dst")
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
    {
        Ok(f) => f,
        Err(_) => {
            // 文件不存在，创建文件
            let f = File::create(&file_path).expect("无法创建文件");
            // 将内容写入文件
            let mut writer = BufWriter::new(f);
            writer.write_all(contents.as_bytes()).expect("写入文件失败");
            return;
        }
    };

    // 文件存在，在文件尾部追加内容
    file.write_all(b"\n").expect("写入文件失败");
    file.write_all(contents.as_bytes()).expect("写入文件失败");
}
