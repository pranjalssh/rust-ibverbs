use std::time::Duration;

fn main() {
    let ctx = ibverbs::devices()
        .unwrap()
        .iter()
        .next()
        .expect("no rdma device available")
        .open()
        .unwrap();

    let cq = ctx.create_cq(16, 0).unwrap();
    let pd = ctx.alloc_pd().unwrap();

    let qp_builder = pd
        .create_qp(&cq, &cq, ibverbs::ibv_qp_type::IBV_QPT_RC)
        .set_gid_index(1)
        .build()
        .unwrap();

    let endpoint = qp_builder.endpoint().unwrap();
    let mut qp = qp_builder.handshake(endpoint).unwrap();

    let mut mr = pd.allocate::<u64>(2).unwrap();
    mr[1] = 0x42;

    qp.post_receive(&[mr.slice(..1)], 2).unwrap();
    qp.post_send(&[mr.slice(1..)], 1).unwrap();

    let mut sent = false;
    let mut received = false;
    let mut completions = [ibverbs::ibv_wc::default(); 16];
    while !sent || !received {
        let completed = cq
            .wait(&mut completions[..], Some(Duration::from_secs(1)))
            .unwrap();
        assert!(!completed.is_empty());
        assert!(completed.len() <= 2);
        for wr in completed {
            match wr.wr_id() {
                1 => {
                    assert!(!sent);
                    sent = true;
                    println!("sent");
                }
                2 => {
                    assert!(!received);
                    received = true;
                    assert_eq!(mr[0], 0x42);
                    println!("received");
                }
                _ => unreachable!(),
            }
        }
    }
}
