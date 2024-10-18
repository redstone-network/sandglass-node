use crate::{mock::*, Error, *};
use frame_support::{assert_noop, assert_ok, traits::fungible::Inspect};
use orml_traits::MultiCurrency;
use primitives::currency::{CurrencyId, TokenSymbol};
use sp_core::U256;

#[test]
fn test_setup_verification() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", Some("3701847203724321478317961353917758270528478504282408535117312363800157867784070247396381164448597370877483548917602".to_owned()));
		assert_ok!(MixerModule::setup_verification(
			RuntimeOrigin::signed(1),
			vk.as_bytes().into()
		));

	});
}

#[test]
fn test_deposit() {
	new_test_ext().execute_with(|| {
		let before = Balances::balance(&1);
		assert_ok!(MixerModule::deposit(RuntimeOrigin::signed(1), vec![1]));
		let after = Balances::balance(&1);

		assert_eq!(before, after + 1_000);

		let c = U256::from_big_endian(&vec![1]);
		assert_eq!(Commitments::<Test>::contains_key(c), true);

		assert_eq!(MerkleVec::<Test>::get(), vec![c]);

		let root = U256::from_dec_str(
			"11918823777688916996440235409179584458198237132535057418448191606750426488941",
		)
		.unwrap();
		assert_eq!(Roots::<Test>::contains_key(root), true);
	});
}

#[test]
fn test_withdraw() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		assert_ok!(MixerModule::setup_verification(RuntimeOrigin::signed(1), vk.as_bytes().into()));

		let com = U256::from_dec_str(
			"21230726593955799921294235288119148384132470247301542251439992487769590832624",
		)
		.unwrap();
		let mut com_bytes = [0u8; 32];
		com.to_big_endian(&mut com_bytes);

		assert_ok!(MixerModule::deposit(RuntimeOrigin::signed(1), com_bytes.to_vec()));

		assert_noop!(
			MixerModule::withdraw(RuntimeOrigin::signed(1), vec![1], vec![1], vec![1], 2),
			Error::<Test>::MalformedProof
		);

		let root = U256::from_dec_str(
			"11707923398010884771104902347581583507748139574064506485019337720597328298281",
		)
		.unwrap();

		let mut root_bytes = [0u8; 32];
		root.to_big_endian(&mut root_bytes);

		let incorrect_proof = prepare_incorrect_proof_json("groth16", "bls12381", None);

		assert_noop!(
			MixerModule::withdraw(
				RuntimeOrigin::signed(1),
				incorrect_proof.as_bytes().into(),
				root_bytes.to_vec(),
				vec![1u8],
				2
			),
			Error::<Test>::ProofCreationError
		);

		let proof = prepare_proof_json("groth16", "bls12381", None);

		let nullifier = U256::from_dec_str(
			"25552435442991663747900835940687199996982543695763714737597223653118621902822",
		)
		.unwrap();

		let mut nullifier_bytes = [0u8; 32];
		nullifier.to_big_endian(&mut nullifier_bytes);

		let before = Balances::balance(&2);
		assert_ok!(MixerModule::withdraw(
			RuntimeOrigin::signed(1),
			proof.as_bytes().into(),
			root_bytes.to_vec(),
			nullifier_bytes.to_vec(),
			2
		),);
		let after = Balances::balance(&2);
		assert_eq!(before + 1_000, after);

		assert_noop!(
			MixerModule::withdraw(
				RuntimeOrigin::signed(1),
				proof.as_bytes().into(),
				root_bytes.to_vec(),
				nullifier_bytes.to_vec(),
				2
			),
			Error::<Test>::NoteHasBeanSpent
		);
	});
}

#[test]
fn test_swap() {
	new_test_ext().execute_with(|| {
		let vk = prepare_vk_json("groth16", "bls12381", None);
		assert_ok!(MixerModule::setup_verification(RuntimeOrigin::signed(1), vk.as_bytes().into()));

		let com = U256::from_dec_str(
			"21230726593955799921294235288119148384132470247301542251439992487769590832624",
		)
		.unwrap();
		let mut com_bytes = [0u8; 32];
		com.to_big_endian(&mut com_bytes);

		assert_ok!(MixerModule::deposit(RuntimeOrigin::signed(1), com_bytes.to_vec()));

		let root = U256::from_dec_str(
			"11707923398010884771104902347581583507748139574064506485019337720597328298281",
		)
		.unwrap();

		let mut root_bytes = [0u8; 32];
		root.to_big_endian(&mut root_bytes);

		let proof = prepare_proof_json("groth16", "bls12381", None);

		let nullifier = U256::from_dec_str(
			"25552435442991663747900835940687199996982543695763714737597223653118621902822",
		)
		.unwrap();

		let mut nullifier_bytes = [0u8; 32];
		nullifier.to_big_endian(&mut nullifier_bytes);

		assert_eq!(Currencies::free_balance(CurrencyId::VToken(TokenSymbol::BTC), &2), 5_000);
		assert_eq!(Currencies::free_balance(CurrencyId::VToken(TokenSymbol::BTC), &3), 5_000);
		assert_eq!(Balances::balance(&3), 5000);

		assert_ok!(Swap::submit_order(
			RuntimeOrigin::signed(3),
			CurrencyId::VToken(TokenSymbol::BTC),
			10,
			CurrencyId::Token(TokenSymbol::DOT),
			1000
		));

		assert_ok!(MixerModule::swap(
			RuntimeOrigin::signed(1),
			proof.as_bytes().into(),
			root_bytes.to_vec(),
			nullifier_bytes.to_vec(),
			0,
			2,
		),);

		assert_eq!(Currencies::free_balance(CurrencyId::VToken(TokenSymbol::BTC), &2), 5_010);
		assert_eq!(Balances::balance(&3), 6000);
	});
}

#[test]
fn test_blacklist() {
	new_test_ext().execute_with(|| {
        assert_ok!(MixerModule::add_black_list(RuntimeOrigin::signed(1), 1));

        let vk = prepare_vk_json("groth16", "bls12381", Some("3701847203724321478317961353917758270528478504282408535117312363800157867784070247396381164448597370877483548917602".to_owned()));
		assert_ok!(MixerModule::setup_verification(
			RuntimeOrigin::signed(1),
			vk.as_bytes().into()
		));

		assert_noop!(
            MixerModule::deposit(RuntimeOrigin::signed(1),vec![1]),
            Error::<Test>::BlacklistRejected
        );

        assert_noop!(
			MixerModule::withdraw(RuntimeOrigin::signed(1), vec![1], vec![1], vec![1], 2),
			Error::<Test>::BlacklistRejected
		);

	});
}

fn _prepare_correct_public_inputs_json() -> String {
	r#"[
        "12154017155188732043720388494527814426846884333686418648942396484836291069935",
        "47383248954783409320757252323368067485491150229432134318939482346666131919279"
]"#
	.to_owned()
}

fn _prepare_incorrect_public_inputs_json() -> String {
	r#"[
        "3"
]"#
	.to_owned()
}

fn _prepare_empty_public_inputs_json() -> String {
	r#"[
]"#
	.to_owned()
}

//alpha_x is
fn prepare_vk_json(protocol: &str, curve: &str, alpha_x: Option<String>) -> String {
	let alpha_x = alpha_x.unwrap_or_else(|| "2448874857023974973026039179601311941245479845877357215799007016607896985573141278772848013541221206695622401150255".to_owned());
	let vk_template = r#"{
"protocol": "<protocol>",
"curve": "<curve>",
"nPublic": 2,
"vk_alpha_1": [
   "<alpha_x>",
   "1356372945095092488355761279959555937533398384628269592987610253636932229929493086101626441995479664167971984880995",
  "1"
 ],
 "vk_beta_2": [
  [
   "2883557283179017990684231795368683826531564748523328167192891173597163310626361843827650299706150752112128247781718",
   "2817824425687704468001957429962220025241283972312504240277526642453565933302630881962254706181115007482924051409656"
  ],
  [
   "1944853843526887003804921136604157095102019030361151660248118265420014854066295417794605162911160495829422730839234",
   "3863387530621888759020457725754991180837923423490110062569036152805411815619492931752796804028413687263808178841259"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_gamma_2": [
  [
   "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160",
   "3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758"
  ],
  [
   "1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905",
   "927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_delta_2": [
  [
   "54914076621958694751565355897841036335765064662226446795997338252566358370434592752110703814310071896267850983640",
   "1999244175769498267777836540053722732768863305916601299372023231188477238261626833666950286528737143011262370343540"
  ],
  [
   "1909672756823837448174089300349973261563914576075977272280724719112387445660411586242074181066802602997851100183323",
   "696541802754615100390907730659808157479256236674397300950801911856627800479869251507555183409654546713990344031816"
  ],
  [
   "1",
   "0"
  ]
 ],
 "vk_alphabeta_12": [
  [
   [
    "326064032597870119633781736439074538637774398296266905216339777620003381197081644497355777914208966151111209392665",
    "3655920772032021274406953280738586525649440986780878509190598352248047281913537085704306605923669612820230206967573"
   ],
   [
    "3035160534927130191078271348783363481261573837733881604766479325099231182780499775702254097498996595439612239463020",
    "190279643963606395670502738846837140904233096982535910918610683581499978303421574649693084438139924764850347656741"
   ],
   [
    "2887858830158727788900555777821267062275477073110592253587430625686135760810478452850202305334838021567517266989612",
    "2843412311245475436233513642502442596960150485241567407373819482095697089972394374435584822008137756353517499816050"
   ]
  ],
  [
   [
    "3403598786870643138427041327140917777436749165429629454726414734513849857817773549785223306216733206379404447295327",
    "3503678643230755378831424324614754210165116731730070992535006838785679119682646166763881872424204340705680718757442"
   ],
   [
    "3220053787852403718819854407615602718099504704298448521397835642294870904545862748860945204799053513167217996039778",
    "2640762678347347143689267130660332287883445885258029212100341723510134671017278476100472949271365374279128013565350"
   ],
   [
    "3141149540718016616010422111936487913919537151165018920635597997648553334761512497917250515350769152770025335174734",
    "2406143936460283403894419403869784263547828156081066959285667421782953950736333395255229104907662106711112075779502"
   ]
  ]
 ],
 "IC": [
  [
   "3238616715122989247135112038747018645755882381443984262429221074158301207925618704135276853096498558292425291298242",
   "3992154954145077585643960524400012910595371098612940577232633132480876457042234861152139921436311154130720893607825",
   "1"
  ],
  [
   "1317981778966324232395763408436972705300309762012561797246179374393524029788979897324138254778596876285402066363409",
   "1012027418433913400221466281217360011303276349417091621012324221371259134763931112976087141357038786144091665401078",
   "1"
  ],
  [
   "719904090459766113556465036896653198312710565196233141421845008457589324016290858586672065118873586748107301659121",
   "1017436267077909227073507398373091806172200485980954785569410477214171517010439488135906388407984649924316175712374",
   "1"
  ]
 ]
}"#;
	vk_template
		.replace("<protocol>", protocol)
		.replace("<curve>", curve)
		.replace("<alpha_x>", &alpha_x)
}

// input is
// {
// 	"root": "11707923398010884771104902347581583507748139574064506485019337720597328298281",
// 	"nullifierHash":
// "25552435442991663747900835940687199996982543695763714737597223653118621902822", 	"secret":
// "15308241608160268350", 	"paths2_root": ["0",
// "51576823595707970152643159819788304363803754756066229172775779360774743019614",
// "33646187916922823865935622258451714952164674255482660942215703235411158105736",
// "27818645450144846908742692719385898720249207574255739267233226464286012246073",
// "39404029000907277292464556408734412130261913210564395069696342233560511006152",
// "24907123534309659921713005795092724527532698077589223246276579583330771465031",
// "22103361713848256938655449390262013863291224679776344310249539314760174194771",
// "28665358770471415124367990738618755861132249577405347373337125991381323369983"],
// 	"paths2_root_pos": [0, 0, 0, 0, 0, 0, 0, 0]
// }
fn prepare_proof_json(protocol: &str, curve: &str, pi_a_x: Option<String>) -> String {
	let pi_a_x = pi_a_x.unwrap_or_else(|| "1177564558550769956785339462842115460663322334705567835885914518147582035494482474444088433269065054572726278485401".to_owned());
	let proof_template = r#"{
"pi_a": [
"<pi_a_x>",
    "3463774854543115711511322796130875262715327601213314678256509445986239775532424704720161136511271046778428668115295", "1"],
	"pi_b": [
		["3939735334779363728874826927428882850014215914525443141528712663073265693419464582354918395773933609568093166281375", "2762082853524568870366722466346836656196165120714407758945309012775931831893159584521431747408484009464992517674096"],
		["1238881836365727332955464755544056303340279533650879933373712064446408017638369002208624038950250892465747100623589", "2133327084326797619312560177340933012701302717709822268851609732238727555832689865031979464510335339231088766600526"],
		["1", "0"]
	],
	"pi_c": ["352516744368930426190313000951685856589231636023979896326701901794604226910259816430279862303227124005715360165947", "1207590512348526635165457329307974841887001330405831957414491116032670409554156238728603276568528858063625545112694", "1"],
	
"protocol": "<protocol>",
"curve": "<curve>"
}"#;

	proof_template
		.replace("<protocol>", protocol)
		.replace("<curve>", curve)
		.replace("<pi_a_x>", &pi_a_x)
}

fn prepare_incorrect_proof_json(protocol: &str, curve: &str, pi_a_x: Option<String>) -> String {
	let pi_a_x = pi_a_x.unwrap_or_else(|| "1547868284561670884744470829066291861753711715427536197016979117727657722537367306855408779073400007356480755992286".to_owned());
	let proof_template = r#"{
"pi_a": [
"<pi_a_x>",
    "2928190285687118456628237490794665579453180400888749391929466989435165647115585828297072258065296795029705291422264",
  "1"
 ],
 "pi_b": [
  [
   "1544760086384659722491129944413761133791624948103687884212411089143858975622856940458923129956549037030696758467649",
   "2464775189940920238959991888711294617710220437968413348610396451115323002165937973643445926899349404418574329460530"
  ],
  [
   "3298763698389029215031200968019705888307726502718892947097194556364826276603985042582992623562599586739493850907434",
   "2484185654597531706472773683412570119063675254953464390794474438981035635108861904757019365474913600425823454389195"
  ],
  [
   "1",
   "0"
  ]
 ],
 "pi_c": [
  "3327459656536994775566432277999577617658464546941643592998127983803641389162440694800645493012291115986979472221453",
  "375275461267575649570959031559608367700810891085934218713539805936334713365508835650335023558572824609826972817941",
  "1"
 ],
"protocol": "<protocol>",
"curve": "<curve>"
}"#;

	proof_template
		.replace("<protocol>", protocol)
		.replace("<curve>", curve)
		.replace("<pi_a_x>", &pi_a_x)
}
