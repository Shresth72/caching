#![allow(unused)]
#[cfg(test)]
mod handler_tests {
    use crate::connection;
    use crate::handler::*;

    // Throwing id column not found error
    #[tokio::test]
    async fn _create_works() {
        let state = connection::conn().await.unwrap();

        let body = create::CreateSpellBody {
            name: "Punch".to_string(),
            damage: 90,
        };

        let result = crate::handler::create::create(state, body).await.unwrap();
        assert_eq!(result.name, "Punch");
        assert_eq!(result.damage, 90);
    }

    // #[tokio::test]
    async fn list_works() {
        let state = connection::conn().await.unwrap();
        let result = list::list_spells(state).await.unwrap();
        assert_eq!(result.len(), 1);
    }

}
