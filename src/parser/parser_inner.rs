#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct ParserInner;

#[cfg(test)]
mod test {
    use super::ParserInner;
    use super::Rule;
    use pest::Parser;

    const BASIC_SRC: &str = r#"
        /* This a test */

        faction_set_slot fac.player_supporters_faction slot_faction_state sfs_inactive;

        assign g.player_luck 200;
        assign g.player_luck 200;

        troop_set_slot trp.player slot_troop_occupation slto_kingdom_hero;

        store_random_in_range :starting_training_ground
            training_grounds_begin training_grounds_end;

        party_relocate_near_party p.main_party :training_ground 3;

        str_store_toop_name str.5 trp.player;

        party_set_name p.main_party str.5;

        call_script script.update_party_creation_random_limits;

        assign g.player_party_icon -1;
    "#;

    #[test]
    pub fn basic_test() {
        let _pairs = ParserInner::parse(Rule::main, BASIC_SRC).unwrap();
    }
}
