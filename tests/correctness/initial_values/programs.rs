use super::super::super::*;

#[allow(dead_code)]
#[repr(C)]
#[derive(Default)]
struct Datatype1 {
    field_1: i16,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Default)]
struct Datatype2 {
    field_2: i16,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Default)]
struct Datatype3 {
    field_3: i16,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Default)]
struct Datatype4 {
    field_4: i16,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Default)]
struct MainTypePrgInit {
    b_ret0: bool,
    b_ret1: bool,
}

#[test]
fn prg_init() {
    let function = r"

      TYPE Datatype1 :
      STRUCT
        field_1 : INT := 0;
      END_STRUCT
      END_TYPE
      
      TYPE Datatype2 :
      STRUCT
        field_2 : INT := 0;
      END_STRUCT
      END_TYPE

      TYPE Datatype3 :
      STRUCT
          field_3 : INT := 0;
      END_STRUCT
      END_TYPE

      TYPE Datatype4 :
      STRUCT
        field_4 : INT := 0;
      END_STRUCT
      END_TYPE
  
      TYPE Datatype5 :
      STRUCT
        a_start_point : ARRAY [1..2] OF INT;
        a_point1     : ARRAY [1..2] OF INT;
        a_point2     : ARRAY [1..2] OF INT;
        a_point3     : ARRAY [1..2] OF INT;
        a_point4     : ARRAY [1..2] OF INT;
        a_end_point   : ARRAY [1..2] OF INT;		
      END_STRUCT
      END_TYPE

      PROGRAM program_0
      VAR_INPUT
          in_var1    : Datatype1 := (field_1 := 200);
          in_var2    : Datatype2 := (field_2 := 300);
          in_var3    : Datatype3 := (field_3 := 400);
          in_var4    : Datatype4 := (field_4 := 500);
          in_var5    : Datatype5 := (a_start_point := (3,3), a_point1 := (5,2), a_point2 := (7,3),  
                                      a_point3 := (8,5), a_point4 := (5,7), a_end_point:=(3,5));
      END_VAR
      VAR_OUTPUT
          out_var1    : Datatype1 := (field_1 := 20000);
          out_var2    : Datatype2 := (field_2 := 30000);
          out_var3    : Datatype3 := (field_3 := 40000);
          out_var4    : Datatype4 := (field_4 := 50000);
          out_var5    : Datatype5 := (a_start_point := (3,3), a_point1 := (5,2), a_point2 := (7,3),  
                                    a_point3 := (8,5), a_point4 := (5,7), a_end_point:=(3,5);
          b_ret_val    : BOOL;
      END_VAR
      
      IF in_var1.field_1 <> 200 OR in_var2.field_2 <> 300 OR in_var3.field_3 <> 400 OR in_var4.field_4 <> 500 THEN
          b_ret_val := TRUE;
      ELSIF in_var5.a_start_point[1] <> 3 OR 
            in_var5.a_start_point[2] <> 3 OR 
            in_var5.a_point1[1] <> 5 OR 
            in_var5.a_point1[2] <> 2 OR 
            in_var5.a_point2[1] <> 7 OR 
            in_var5.a_point2[2] <> 3 OR 
            in_var5.a_point3[1] <> 8 OR 
            in_var5.a_point3[2] <> 5 OR 
            in_var5.a_point4[1] <> 5 OR 
            in_var5.a_point4[2] <> 7 OR 
            in_var5.a_end_point[1] <> 3 OR 
            in_var5.a_end_point[2] <> 5 THEN
          b_ret_val := TRUE;
      ELSIF out_var1.field_1 <> 20000 OR out_var2.field_2 <> 30000 OR out_var3.field_3 <> 40000 OR out_var4.field_4 <> 50000 THEN
          b_ret_val := TRUE;
      ELSIF out_var5.a_start_point[1] <> 3 OR 
            out_var5.a_start_point[2] <> 3 OR 
            out_var5.a_point1[1] <> 5 OR 
            out_var5.a_point1[2] <> 2 OR 
            out_var5.a_point2[1] <> 7 OR 
            out_var5.a_point2[2] <> 3 OR 
            out_var5.a_point3[1] <> 8 OR 
            out_var5.a_point3[2] <> 5 OR 
            out_var5.a_point4[1] <> 5 OR 
            out_var5.a_point4[2] <> 7 OR 
            out_var5.a_end_point[1] <> 3 OR 
            out_var5.a_end_point[2] <> 5 THEN
          b_ret_val := TRUE;
      END_IF
      
      END_PROGRAM

      PROGRAM main
      VAR
        b_ret0    : BOOL;
        b_ret1    : BOOL;
    END_VAR

    program_0();

    b_ret0 := program_0.b_ret_val;

    IF program_0.out_var1.field_1 <> 20000 OR program_0.out_var2.field_2 <> 30000 OR program_0.out_var3.field_3 <> 40000 OR program_0.out_var4.field_4 <> 50000 THEN
        b_ret1 := TRUE;
    ELSIF program_0.out_var5.a_start_point[1] <> 3 OR 
          program_0.out_var5.a_start_point[2] <> 3 OR 
          program_0.out_var5.a_point1[1] <> 5 OR 
          program_0.out_var5.a_point1[2] <> 2 OR 
          program_0.out_var5.a_point2[1] <> 7 OR 
          program_0.out_var5.a_point2[2] <> 3 OR 
          program_0.out_var5.a_point3[1] <> 8 OR 
          program_0.out_var5.a_point3[2] <> 5 OR 
          program_0.out_var5.a_point4[1] <> 5 OR 
          program_0.out_var5.a_point4[2] <> 7 OR 
          program_0.out_var5.a_end_point[1] <> 3 OR 
          program_0.out_var5.a_end_point[2] <> 5 THEN
        b_ret1 := TRUE;
    END_IF


    END_PROGRAM
      ";

    let mut new_prg_init: MainTypePrgInit = MainTypePrgInit::default();

    compile_and_run::<_, i32>(function.to_string(), &mut new_prg_init);

    assert_eq!(false, new_prg_init.b_ret0);
    assert_eq!(false, new_prg_init.b_ret1);
}
