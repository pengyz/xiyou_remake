    use super::scene_test_support::scene;

    /// BUG-054 红钉：使用下行符（型 22）=changeFloor(currentFloor−1,true,false)
    /// +keyHeld=0+gameMode=3（a.java:7868-7872 case22 逐字）。走 use_item_stack
    /// 真实使用面（a.java:7761-7781；面板仅 13/14 直发，其余经确认浮层 −6 臂）。
    #[test]
    fn bug054_use_down_stair_decrements_floor() {
        let mut sc = scene((2, 2), &[(43, 5, 5)]);
        sc.persist.floor = 10;
        sc.player.item_types[0] = 22;
        sc.player.item_uses[0] = 1;
        sc.player.item_stack_size = 1;
        sc.use_item_stack(0); // 型22：activate_item(22)→changeFloor(floor−1)
        assert_eq!(sc.persist.pending_floor, 9, "使用下行符=changeFloor(floor−1)（BUG-054，pending=9）");
        assert!(sc.persist.went_down, "went_down=true（a.java:7869 down 参）");
        assert_eq!(sc.key_held, 0, "keyHeld=0（a.java:7870）");
    }
