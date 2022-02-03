use core::panic;

use crate::{assert_type_and_hint, ast::{AstStatement, ConditionalBlock}, test_utils::tests::index, TypeAnnotator, typesystem::DINT_TYPE};

#[test]
fn binary_expressions_resolves_types() {
    let (unit, index) = index(
        "PROGRAM PRG
                VAR x : INT; END_VAR
                FOR x := 3 TO 10 BY 2 DO 
                    x;
                END_FOR
        END_PROGRAM",
    );
    let (annotations, _) = TypeAnnotator::visit_unit(&index, &unit);
    let statements = &unit.implementations[0].statements;

    if let AstStatement::ForLoopStatement {
        counter,
        start,
        end,
        by_step: Some(by_step),
        ..
    } = &statements[0]
    {
        assert_type_and_hint!(&annotations, &index, counter, "INT", None);
        assert_type_and_hint!(&annotations, &index, start, "DINT", Some("INT"));
        assert_type_and_hint!(&annotations, &index, end, "DINT", Some("INT"));
        assert_type_and_hint!(&annotations, &index, by_step, "DINT", Some("INT"));
    } else {
        panic!("no for loop statement");
    }
}

#[test]
fn bool_expression_in_if_annotation() {
    //GIVEN
    let (unit, index) = index(
        "
        PROGRAM PRG
		VAR
			a,b : BOOL;
		END_VAR
            IF (a OR B) THEN
            END_IF
        END_PROGRAM
        ",
    );

    //WHEN the AST is annotated
    let (annotations, _) = TypeAnnotator::visit_unit(&index, &unit);
    
    let if_a_or_b = &unit.implementations[0].statements[0];

    // THEN all parts of the expression should be annotated with BOOL
    if let AstStatement::IfStatement{ blocks: conditions, ..} = if_a_or_b{
        assert_type_and_hint!(&annotations, &index, &conditions[0].condition, DINT_TYPE, None);
    }else{
        unreachable!("expected binary expression")
    }
}