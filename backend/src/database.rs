use mysql_async::prelude::*;
use mysql_async::Pool;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub fn new(url: &str) -> Self {
        Self {
            pool: Pool::new(url),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Nursery {
    pub name: String,
    pub url: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: usize,
    pub miles: usize,
}

impl Database {
    pub async fn find_nurseries(&self, zip: &str) -> Vec<Nursery> {
        let mut conn = self.pool.get_conn().await.unwrap();

        r"
SELECT miles, name, url, address, city, state, n.zipcode
FROM zipcodes_nurseries zn
INNER JOIN nurseries n 
  ON n.id = zn.nursery_id 
WHERE zn.zipcode = ?
ORDER BY miles ASC"
            .with((zip,))
            .map(
                &mut conn,
                |(miles, name, url, address, city, state, zip)| Nursery {
                    name,
                    url,
                    address,
                    city,
                    state,
                    zip,
                    miles,
                },
            )
            .await
            .unwrap()
    }
}
