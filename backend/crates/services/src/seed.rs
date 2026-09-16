use chrono::{Duration, Utc};
use common::AppError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use db::entities::{
    creator_profile as profile_entity, service as service_entity, user as user_entity,
    work as work_entity,
};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

struct SeedCreator {
    username: &'static str,
    nickname: &'static str,
    avatar: &'static str,
    intro: &'static str,
    bio: &'static str,
    rating: Decimal,
    level: &'static str,
    city: &'static str,
    service_title: &'static str,
    service_price: Decimal,
    service_duration: i32,
    service_type: &'static str,
    works: &'static [(&'static str, &'static str, &'static str, &'static str)],
}

fn creators() -> Vec<SeedCreator> {
    vec![
    SeedCreator {
        username: "chenyu",
        nickname: "陈屿",
        avatar: "/demo/works/portrait-1.jpg",
        intro: "专注于自然光人像，擅长捕捉情绪与松弛感。",
        bio: "十年人像经验，服务超过 300 位客人。",
        rating: Decimal::new(49, 1),
        level: "elite",
        city: "杭州",
        service_title: "城市自然光人像写真",
        service_price: Decimal::new(3999_00, 2),
        service_duration: 120,
        service_type: "人像摄影",
        works: &[
            ("/demo/works/portrait-1.jpg", "晨光里的侧影", "人像", "自然光，安静的情绪。"),
            ("/demo/works/portrait-2.jpg", "白裙与风", "人像", "外景人像，松弛的肢体。"),
            ("/demo/works/portrait-4.jpg", "金色时刻", "人像", "黄昏逆光人像。"),
        ],
    },
    SeedCreator {
        username: "linye",
        nickname: "林野",
        avatar: "/demo/works/street-1.jpg",
        intro: "街头纪实摄影师，记录城市的呼吸与温度。",
        bio: "穿梭在城市街头，寻找被忽略的瞬间。",
        rating: Decimal::new(48, 1),
        level: "premium",
        city: "上海",
        service_title: "城市街拍纪实",
        service_price: Decimal::new(2599_00, 2),
        service_duration: 90,
        service_type: "人像摄影",
        works: &[
            ("/demo/works/street-1.jpg", "十字路口", "街拍", "城市街头的流动。"),
            ("/demo/works/street-2.jpg", "清晨的街区", "街拍", "光影与建筑的对话。"),
            ("/demo/works/night-1.jpg", "夜未央", "街拍", "霓虹下的夜色。"),
        ],
    },
    SeedCreator {
        username: "suhe",
        nickname: "苏禾",
        avatar: "/demo/works/wedding-1.jpg",
        intro: "婚礼纪实摄影师，用镜头留住真实的仪式感。",
        bio: "拍过 200 场婚礼，擅长捕捉不被安排的瞬间。",
        rating: Decimal::new(49, 1),
        level: "elite",
        city: "厦门",
        service_title: "婚礼全天纪实跟拍",
        service_price: Decimal::new(8888_00, 2),
        service_duration: 480,
        service_type: "婚礼纪实",
        works: &[
            ("/demo/works/wedding-1.jpg", "交换誓言", "婚礼", "真实又动人的仪式。"),
            ("/demo/works/wedding-2.jpg", "手捧花", "婚礼", "细节里的温柔。"),
        ],
    },
    SeedCreator {
        username: "zhoumo",
        nickname: "周墨",
        avatar: "/demo/works/commercial-1.jpg",
        intro: "商业静物与品牌视觉摄影师。",
        bio: "服务过咖啡、美妆、数码等品牌。",
        rating: Decimal::new(47, 1),
        level: "premium",
        city: "深圳",
        service_title: "产品与品牌视觉拍摄",
        service_price: Decimal::new(5200_00, 2),
        service_duration: 180,
        service_type: "商业拍摄",
        works: &[
            ("/demo/works/commercial-1.jpg", "相机与光", "商业", "精密的产品质感。"),
            ("/demo/works/commercial-2.jpg", "陈列美学", "商业", "品牌空间的秩序。"),
            ("/demo/works/editorial-1.jpg", "质感静物", "商业", "克制的高级感。"),
        ],
    },
    SeedCreator {
        username: "guchuan",
        nickname: "顾川",
        avatar: "/demo/works/travel-1.jpg",
        intro: "旅行与风光摄影师，追逐山川湖海。",
        bio: "自驾走过大半个中国，记录自然的辽阔。",
        rating: Decimal::new(48, 1),
        level: "standard",
        city: "成都",
        service_title: "户外旅拍跟拍",
        service_price: Decimal::new(4500_00, 2),
        service_duration: 360,
        service_type: "短片影像",
        works: &[
            ("/demo/works/travel-1.jpg", "远山", "旅行", "辽阔的山脉。"),
            ("/demo/works/travel-2.jpg", "湖面晨雾", "旅行", "静谧的自然。"),
            ("/demo/works/travel-3.jpg", "云端公路", "旅行", "自由的前方。"),
        ],
    },
    SeedCreator {
        username: "xuyan",
        nickname: "许言",
        avatar: "/demo/works/editorial-2.jpg",
        intro: "时尚与妆造摄影，偏爱干净的色彩与造型。",
        bio: "与多家杂志和造型师合作。",
        rating: Decimal::new(46, 1),
        level: "standard",
        city: "北京",
        service_title: "时尚造型写真",
        service_price: Decimal::new(6800_00, 2),
        service_duration: 240,
        service_type: "人像摄影",
        works: &[
            ("/demo/works/editorial-2.jpg", "光影肖像", "时尚", "克制的张力。"),
            ("/demo/works/makeup-1.jpg", "妆面特写", "时尚", "精致的妆造。"),
            ("/demo/works/makeup-2.jpg", "色彩实验", "时尚", "大胆的配色。"),
        ],
    },
    SeedCreator {
        username: "shenlu",
        nickname: "沈鹿",
        avatar: "/demo/works/food-1.jpg",
        intro: "美食摄影与内容创作，让食物会讲故事。",
        bio: "服务餐饮品牌的菜单与社媒内容。",
        rating: Decimal::new(47, 1),
        level: "standard",
        city: "广州",
        service_title: "美食与菜单摄影",
        service_price: Decimal::new(3200_00, 2),
        service_duration: 180,
        service_type: "商业拍摄",
        works: &[
            ("/demo/works/food-1.jpg", "餐桌上的仪式", "美食", "诱人的色彩。"),
            ("/demo/works/food-2.jpg", "清爽一餐", "美食", "自然的光线。"),
        ],
    },
    SeedCreator {
        username: "hanche",
        nickname: "韩澈",
        avatar: "/demo/works/arch-1.jpg",
        intro: "建筑与空间摄影师，研究结构与光影。",
        bio: "专注建筑、酒店与室内空间摄影。",
        rating: Decimal::new(45, 1),
        level: "standard",
        city: "上海",
        service_title: "建筑与空间摄影",
        service_price: Decimal::new(5800_00, 2),
        service_duration: 240,
        service_type: "商业拍摄",
        works: &[
            ("/demo/works/arch-1.jpg", "几何立面", "建筑", "秩序与光影。"),
        ],
    },
    ]
}

pub async fn seed_demo_data(db: &DatabaseConnection) -> Result<(), AppError> {
    let existing = profile_entity::Entity::find()
        .one(db)
        .await
        .map_err(AppError::from_anyhow)?;
    if existing.is_some() {
        return Ok(());
    }

    let types = db::entities::service_type::Entity::find()
        .all(db)
        .await
        .map_err(AppError::from_anyhow)?;

    let now = Utc::now();
    // 管理员账号
    user_entity::ActiveModel {
        username: Set("admin".into()),
        password_hash: Set(hash("admin123")),
        nickname: Set("平台管理员".into()),
        avatar_url: Set(None),
        status: Set("active".into()),
        role: Set("admin".into()),
        verification_status: Set("verified".into()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(AppError::from_anyhow)?;

    // 普通客户账号
    user_entity::ActiveModel {
        username: Set("customer".into()),
        password_hash: Set(hash("customer123")),
        nickname: Set("林小满".into()),
        avatar_url: Set(None),
        status: Set("active".into()),
        role: Set("user".into()),
        verification_status: Set("unverified".into()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(AppError::from_anyhow)?;

    for (index, creator) in creators().iter().enumerate() {
        let now = Utc::now();
        let user = user_entity::ActiveModel {
            username: Set(creator.username.to_string()),
            password_hash: Set(hash("creator123")),
            nickname: Set(creator.nickname.to_string()),
            avatar_url: Set(Some(creator.avatar.to_string())),
            status: Set("active".into()),
            role: Set("user".into()),
            verification_status: Set("verified".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(AppError::from_anyhow)?;

        let profile = profile_entity::ActiveModel {
            user_id: Set(user.id),
            introduction: Set(Some(creator.intro.to_string())),
            bio: Set(Some(creator.bio.to_string())),
            rating: Set(creator.rating),
            certification_level: Set(creator.level.to_string()),
            service_areas: Set(None),
            available_slots: Set(None),
            style_vector_id: Set(None),
            portfolio_url: Set(None),
            total_services: Set(1),
            total_appointments: Set(0),
            total_income: Set(Decimal::ZERO),
            avg_rating: Set(creator.rating),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(AppError::from_anyhow)?;

        let type_id = types
            .iter()
            .find(|t| t.name == creator.service_type)
            .map(|t| t.id)
            .unwrap_or(1);
        service_entity::ActiveModel {
            creator_id: Set(profile.id),
            type_id: Set(type_id),
            title: Set(creator.service_title.to_string()),
            description: Set(Some(creator.intro.to_string())),
            price: Set(creator.service_price),
            duration: Set(Some(creator.service_duration)),
            cover_image_url: Set(Some(creator.works[0].0.to_string())),
            location: Set(Some(creator.city.to_string())),
            tags: Set(Some(creator.service_type.to_string())),
            options: Set(None),
            is_active: Set(true),
            is_featured: Set(index < 4),
            appointments_count: Set(0),
            style_vector_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(AppError::from_anyhow)?;

        for (i, (image, title, category, description)) in creator.works.iter().enumerate() {
            let created_at = now - Duration::hours(i as i64);
            work_entity::ActiveModel {
                user_id: Set(user.id),
                portfolio_id: Set(None),
                image_url: Set(image.to_string()),
                title: Set(Some(title.to_string())),
                description: Set(Some(description.to_string())),
                category: Set(Some(category.to_string())),
                created_at: Set(created_at),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(AppError::from_anyhow)?;
        }
    }

    Ok(())
}

fn hash(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .expect("argon2 hash")
}
