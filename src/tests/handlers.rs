#![allow(unused)]
#[cfg(test)]
mod handler_tests {
    use crate::connection;
    use crate::handler::*;

    // Throwing id column not found error
    #[tokio::test]
    async fn create_works() {
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
        assert_eq!(result[result.len() - 1].name, "Punch");
    }

    #[tokio::test]
    async fn read_works() {
        let state = connection::conn().await.unwrap();
        let result = read::find_by_id(state, 1).await.unwrap();
        assert_eq!(result.unwrap().name, "Fireball");
    }

    #[tokio::test]
    async fn update_works() {
        let state = connection::conn().await.unwrap();
        let body = update::UpdateBody { damage: 100 };
        let result = update::update(state, 1, body).await.unwrap();
        assert_eq!(result.unwrap().damage, 100);
    }

    #[tokio::test]
    async fn delete_works() {
        let state = connection::conn().await.unwrap();
        let list = list::list_spells(state.clone()).await.unwrap();
        assert_eq!(list[list.len() - 1].name, "Punch");

        let last_id = list[list.len() - 1].id;

        let result = delete::delete_spell(state, last_id).await.unwrap();
        assert_eq!(result, 1);
    }
}
