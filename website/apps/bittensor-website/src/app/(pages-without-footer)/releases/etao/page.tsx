import FadeInWrapper from '@/app/components/FadeInWrapper';
import {Link} from '@raofoundation/ui';
import type {Metadata} from 'next';
import {Suspense} from 'react';
import styles from '../v436-upgrade/page.module.css';

export const metadata: Metadata = {
  title: 'Etao — Subnets Choose Their Emission Split',
  description:
    'Etao (runtime v480) gives every subnet owner a five-slot emission weight vector: how much ' +
    'of each block goes to the owner, to validators, and to miners. Root TAO stakers keep their ' +
    'baseline payout no matter what the owner picks, because every change re-solves the ' +
    "subnet's effective TAO weight once, at set time, against a fresh baseline. One " +
    'extrinsic, one weekly rate limit, five rules, zero change to payouts at upgrade.',
  alternates: {canonical: '/releases/etao'},
  openGraph: {images: '/images/og_thumbs/etao.png'},
};

const DocLink = ({href, children}: {href: string; children: React.ReactNode}) => (
  <Link href={href} className={styles.inline_link}>
    {children}
  </Link>
);

const page = () => {
  return (
    <Suspense fallback={<div style={{minHeight: '100vh', backgroundColor: 'white'}} />}>
      <FadeInWrapper className={styles.page_container}>
        <section className={styles.title_section}>
          <h1 className={styles.paper_title}>Etao</h1>
          <p className={styles.subtitle} style={{fontSize: '10px'}}>
            Subnets choose their emission split · Runtime v480 · October 2026
          </p>
        </section>

        <section className={styles.section}>
          <img
            className={styles.graph}
            src='/images/content/etao-emission-split.svg'
            alt="Two pie charts. Before Etao: 18% owner, 41% validators with root TAO stakers taking 5.4% of emission from that slot, 41% miners. With Etao, an SN4-style subnet lowers validators to 20% and raises miners to 62%; root's 5.4% slice is unchanged because the subnet's TAO weight moves from 18% to 43.9%."
          />
          <p className={styles.graph_caption}>
            Before Etao every subnet paid the same fixed split. With Etao the owner moves the dial —
            here an SN4-style subnet pays miners 62% instead of 41% — and root TAO stakers&apos;
            slice of the pie does not move. Illustrative numbers.
          </p>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>Introduction</h2>
          <p>
            <strong>Etao</strong> hands every subnet owner the dial that has been welded shut since
            launch: how the subnet&apos;s alpha emission is divided. Until now every subnet paid the
            same split — 18% to the owner, 41% to validators, 41% to miners. From spec{' '}
            <strong>480</strong> a subnet that lives on its miners can pay them 62%. A subnet that
            needs deep validation can push validators to the 50% cap. An owner who wants to signal
            commitment can set their own cut to zero.
          </p>
          <p>
            One extrinsic, one rate limit, five rules, and a guarantee underneath all of it:{' '}
            <strong>
              root TAO stakers are paid exactly what they would be on an untouched subnet, whatever
              the owner picks.
            </strong>{' '}
            The guarantee is not a promise; it is arithmetic the chain runs at every change.
          </p>
          <p>
            Etao ships to mainnet as a sudo runtime upgrade from <code>main</code>. Payouts in the
            first post-upgrade block are identical to the last pre-upgrade block; nothing moves
            until an owner moves it. The complete, standalone release note — every rule, error,
            storage item and worked CLI example — is the{' '}
            <DocLink href='/docs/guides/etao'>Etao release guide</DocLink>.
          </p>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>Why</h2>
          <p>
            Subnets are not alike. Some are miner-heavy compute markets where validation is cheap
            and the scarce resource is miner effort. Some are judgment-heavy and need validators to
            spend real money to score work. Some are owner-run products where the owner cut funds
            the team; others want that cut gone so every token flows to participants. A single fixed
            split cannot serve all of them, and until now the only knob — the owner cut — was a
            global sudo setting, the same for every subnet.
          </p>
          <p>
            The obstacle to letting owners move the validator share was root. Root TAO stakers are
            paid from the validator slot of every subnet. Cut that slot from 41% to 20% and, with
            nothing else changing, root&apos;s income from the subnet would halve: owners could tax
            a network-wide stakeholder to pay their own participants. Etao removes that coupling.
            The validator slot becomes a choice about <em>validators</em>, and root&apos;s payout is
            pinned to a baseline the owner cannot move.
          </p>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>The emission weight vector</h2>
          <p>
            Every subnet now carries a five-slot vector{' '}
            <code>[burn, recycle, owner, validator, miner]</code>. Each slot is a <code>u64</code>{' '}
            share of <code>u64::MAX</code>, so the five slots sum to exactly one with no rounding
            dust. Per block, <code>alpha_out × owner</code> accrues to the owner,{' '}
            <code>alpha_out × validator</code> to the validator bucket and{' '}
            <code>alpha_out × miner</code> to the miner bucket.
          </p>
          <table className={styles.metrics_table}>
            <thead>
              <tr>
                <th>Slot</th>
                <th>Paid to</th>
                <th>Rule in v1</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>
                  <code>burn</code>
                </td>
                <td>nothing — alpha is destroyed</td>
                <td>must be 0 (reserved)</td>
              </tr>
              <tr>
                <td>
                  <code>recycle</code>
                </td>
                <td>nothing — alpha returns to the unissued pool</td>
                <td>must be 0 (reserved)</td>
              </tr>
              <tr>
                <td>
                  <code>owner</code>
                </td>
                <td>the subnet owner&apos;s hotkey</td>
                <td>0% to 33%; lease-created subnets at least 18%</td>
              </tr>
              <tr>
                <td>
                  <code>validator</code>
                </td>
                <td>validator dividends (root TAO stakers are paid from this slot)</td>
                <td>at most 50%, and high enough that root&apos;s baseline share is reachable</td>
              </tr>
              <tr>
                <td>
                  <code>miner</code>
                </td>
                <td>miner incentive</td>
                <td>at least 33%</td>
              </tr>
            </tbody>
          </table>
          <p>
            The setter is{' '}
            <DocLink href='/docs/tx/set-subnet-emission-weights'>
              <code>AdminUtils.sudo_set_subnet_emission_weights(netuid, uids, values)</code>
            </DocLink>
            . It is sparse: name the slots you want to change and their new shares; the others keep
            their value. The merged vector must pass every rule, and each rule has its own error so
            a rejection names the constraint. Callers are the subnet owner coldkey or root. Owners
            may change a subnet&apos;s vector <strong>once every 50,400 blocks</strong> (about seven
            days). The limit is keyed by subnet, not by coldkey, so handing a subnet to a new owner
            does not reset it; a subnet&apos;s first change is free, and root is exempt.
          </p>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>How root stays whole</h2>
          <p>
            Root TAO is weighted against alpha by a TAO weight <code>w</code>. Each block, the alpha
            root stakers receive from a subnet is
          </p>
          <pre
            className={styles.code_block}
          >{`root_alpha = V × x / (x + A) × alpha_out        with x = root_tao × w`}</pre>
          <p>
            where <code>V</code> is the validator slot and <code>A</code> the subnet&apos;s alpha
            issuance. Before Etao, <code>w</code> was the single global <code>TaoWeight</code> and{' '}
            <code>V</code> was fixed everywhere.
          </p>
          <p>
            Etao gives every subnet its own <strong>effective TAO weight</strong> <code>w_s</code> (
            <code>SubnetTaoWeight</code>), used on the consensus side of that subnet only: the root
            share carved out of the validator slot, the TAO-versus-alpha split of validator
            dividends, and the stake weighting inside Yuma Consensus. Every accepted set call solves{' '}
            <code>w_s</code> <strong>once</strong>, at that block, from live chain values and the
            new validator slot alone:
          </p>
          <pre className={styles.code_block}>{`# 1. root's baseline share of this subnet's emission
r     = V0 × x_g / (x_g + A)      x_g = root_tao × TaoWeight (global)
                                   V0  = validator share of the default vector (0.41)

# 2. the weight that pays r under the new validator slot
x_new = r × A / (V_new − r)
w_s   = x_new / root_tao

# rejected with ValidatorEmissionTooLowToPreserveRoot if V_new ≤ r or w_s > 1
# equivalently, the validator floor at this block is
V_min = r × (1 + A / root_tao)`}</pre>
          <p>
            So the block after any change, root TAO stakers earn from the subnet exactly what an
            untouched subnet would pay them at that block. Lowering <code>V</code> raises{' '}
            <code>w_s</code>; raising <code>V</code> lowers it; the alpha root receives does not
            move. Two properties follow from anchoring to a <em>fresh baseline</em> rather than to
            root&apos;s current share:
          </p>
          <ul className={styles.list}>
            <li>
              <strong>No ratchet.</strong> The solve never reads the old vector or the old{' '}
              <code>w_s</code>. An owner who raises <code>V</code> while the subnet is young and
              lowers it after issuance grows — or the reverse — ends up with the same{' '}
              <code>w_s</code> and the same root share as an owner who made the final change in one
              step. Root&apos;s payout is a function of the chain&apos;s state, not of the
              owner&apos;s history.
            </li>
            <li>
              <strong>Back to default means back to global.</strong> Setting <code>V</code> to
              exactly <code>V0</code> clears the per-subnet entry, so the subnet follows the global{' '}
              <code>TaoWeight</code> again. A subnet that has never changed its validator share has
              no entry at all.
            </li>
          </ul>
          <p>
            Between sets <code>w_s</code> is frozen — never recomputed per block. The weight is
            capped at one (root TAO never counts more than 1:1 against alpha), so not every{' '}
            <code>V</code> is reachable: on a mainnet subnet with 5.44M TAO on root, 6.46M alpha
            issued and <code>TaoWeight = 0.18</code>, root&apos;s baseline is about 5.4% of emission
            and the validator slot can go down to about <strong>11.8%</strong>. The floor moves with
            root stake, issuance and the global weight, so <code>btcli subnets emission show</code>{' '}
            prints the live value and every write checks it locally before anything is signed.
          </p>
          <p>
            <strong>What does not change.</strong> The pool side never sees <code>w_s</code>.{' '}
            <code>tao_in</code>, <code>alpha_in</code>, the alpha-injection cap and{' '}
            <code>RootProp</code> use the global <code>TaoWeight</code>, exactly as before. Changing
            a subnet&apos;s vector changes how its alpha is <em>distributed</em>; it does not change
            how much TAO or alpha is <em>injected</em> into its pool, prices, swaps, or the emission
            the subnet receives.
          </p>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>Migration and compatibility</h2>
          <p>
            The upgrade runs one migration, <code>seed_subnet_emission_weights_v1</code>, which
            seeds every live subnet from its legacy owner-cut state so that the first post-upgrade
            block pays out exactly what the last pre-upgrade block did:
          </p>
          <ul className={styles.list}>
            <li>
              <code>owner</code> is the legacy <code>SubnetOwnerCut</code> when{' '}
              <code>OwnerCutEnabled</code> was true, else 0; <code>validator</code> is half the
              remainder; <code>miner</code> is what is left, absorbing the rounding ulp.
            </li>
            <li>
              No <code>SubnetTaoWeight</code> entry is written, so every subnet keeps following the
              global <code>TaoWeight</code> until its owner changes the vector.
            </li>
            <li>
              <code>SubnetCreatedViaLease</code> is set for subnets with a live lease mapping, and{' '}
              <code>SubnetOwnerCutAccrued</code> is seeded to the exact deduction the legacy lock
              refund applied — including subnets already mid-dissolution — so no refund changes.
            </li>
          </ul>
          <p>
            New subnets are seeded with the default vector <code>[0, 0, 18%, 41%, 41%]</code>.
            Dissolving a subnet clears its vector, weight, lease marker, accrued owner cut and
            rate-limit key, so a reused netuid starts clean.
          </p>
          <p>
            <strong>Deprecated extrinsics.</strong> <code>sudo_set_subnet_owner_cut</code> and{' '}
            <code>sudo_set_owner_cut_enabled</code> keep their call indices and signatures but
            always return <code>Deprecated</code>. Set the owner slot — including to zero — through{' '}
            <code>sudo_set_subnet_emission_weights</code> instead. <code>SubnetOwnerCut</code> and{' '}
            <code>OwnerCutEnabled</code> remain readable as legacy views but are no longer written.
          </p>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>New chain surface</h2>
          <ul className={styles.list}>
            <li>
              <strong>Storage:</strong> <code>SubnetEmissionWeights</code> (the vector),{' '}
              <code>SubnetTaoWeight</code> (<code>Option&lt;u64&gt;</code>; absent = follows
              global), <code>SubnetCreatedViaLease</code>, <code>SubnetOwnerCutAccrued</code>.
            </li>
            <li>
              <strong>Event:</strong>{' '}
              <code>
                SubtensorModule.SubnetEmissionWeightsSet {'{'} netuid, weights {'}'}
              </code>{' '}
              with the full post-merge vector.
            </li>
            <li>
              <strong>Rate limit:</strong>{' '}
              <code>OwnerHyperparamUpdate(Hyperparameter::EmissionWeights)</code> keyed by subnet in{' '}
              <code>LastRateLimitedBlock</code>; constant{' '}
              <code>SubnetEmissionWeightsRateLimit = 50_400</code>.
            </li>
            <li>
              <strong>Runtime API:</strong>{' '}
              <code>SubnetInfoRuntimeApi.get_subnet_emission_weights(netuid)</code> (API version 3);{' '}
              <code>get_subnet_hyperparams_v3</code> adds <code>owner_emission</code>,{' '}
              <code>validator_emission</code> and <code>miner_emission</code>.
            </li>
            <li>
              <strong>EVM</strong> (subnet precompile, index 2051):{' '}
              <code>setSubnetEmissionWeights(netuid, uids, values)</code> and{' '}
              <code>getSubnetEmissionWeights(netuid)</code> returning the vector and the lease flag.
            </li>
            <li>
              <strong>Python SDK:</strong> the{' '}
              <DocLink href='/docs/query/subnet-emission-weights'>
                <code>subnet_emission_weights</code>
              </DocLink>{' '}
              read (vector, shares, effective and global TAO weight, root&apos;s current and
              baseline share, the live validator floor, rate-limit state), the{' '}
              <DocLink href='/docs/tx/set-subnet-emission-weights'>
                <code>SetSubnetEmissionWeights</code>
              </DocLink>{' '}
              intent with a structured before/after preview, and the pure rules module{' '}
              <code>bittensor.emission_weights</code>.
            </li>
            <li>
              <strong>CLI:</strong> <code>btcli subnets emission</code> with <code>show</code>,{' '}
              <code>set</code>, <code>set-value</code>, <code>increase</code> and{' '}
              <code>decrease</code>; <code>--json</code> and <code>--dry-run</code> everywhere;
              fixed exit codes (0 done, 1 missing subnet or blocked, 2 rule violation).
            </li>
          </ul>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>For subnet owners</h2>
          <p>Look before you touch. The Allowed table is live for this block:</p>
          <pre className={styles.code_block}>{`btcli subnets emission show --netuid 4

Vector
      owner  18.00%
  validator  41.00%
      miner  41.00%

Root
  root share of emission  5.4015%
     root baseline share  5.4015%
    effective tao weight  18.00%      follows the global TaoWeight (18.00%)
       next owner change  now         first change; owners may change once per 50,400 blocks

Allowed
      owner  0.00% to 33.00%
  validator  11.8093% to 50.0000%   the floor is where root's baseline share needs a TAO weight of 100%
      miner  >= 33.00%`}</pre>
          <p>
            Preview with <code>--dry-run</code>; the plan shows the before/after vector, root&apos;s
            share before and after, and the TAO weight move:
          </p>
          <pre
            className={styles.code_block}
          >{`btcli subnets emission set --netuid 4 --owner 18% --validator 20% --miner 62% --dry-run

dry run: set emission weights on netuid 4: owner=18.00% validator=20.00% miner=62.00%
  effect  vector: owner 18.00% / validator 41.00% / miner 41.00% -> owner 18.00% / validator 20.00% / miner 62.00%
  effect  root earns 5.4015% of this subnet's emission now and 5.4015% after (its baseline:
          default validator share, global TaoWeight); the subnet's TAO weight moves 18.00% -> 43.89%`}</pre>
          <p>
            Then drop <code>--dry-run</code>, or nudge one slot at a time:
          </p>
          <pre
            className={styles.code_block}
          >{`btcli subnets emission increase  --netuid 4 --slot miner --by 5%
btcli subnets emission decrease  --netuid 4 --slot owner --by 0.02
btcli subnets emission set-value --netuid 4 --slot owner --value 0    # owner cut off`}</pre>
          <p>
            A rejected vector exits 2 before anything is signed and names the rule and the allowed
            range:
          </p>
          <pre
            className={styles.code_block}
          >{`error: the chain would reject this change: ValidatorEmissionTooLowToPreserveRoot: validator (slot 3)
       is 5.0000%; ... right now the validator share must be at least about 11.8093%
  note: allowed on subnet 4 right now: validator 11.8093% to 50.0000%
  help: btcli subnets emission show --netuid 4 prints the vector and every allowed range`}</pre>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>Notes by role</h2>
          <ul className={styles.list}>
            <li>
              <strong>Validators.</strong> Your dividends come from the validator slot. Where an
              owner lowers it, the alpha paid to validators and their alpha stakers goes down and
              the subnet&apos;s effective TAO weight goes up, tilting the TAO-versus-alpha split of
              dividends toward TAO stakers. The floor guarantees the slot can never starve root;
              nothing guarantees it stays at 41%. Watch <code>SubnetEmissionWeightsSet</code>{' '}
              events.
            </li>
            <li>
              <strong>Miners.</strong> The miner slot is at least 33% and can grow to whatever is
              left after the owner slot and the validator floor. Weight setting, incentive and the
              epoch are unchanged; only the size of the bucket moves.
            </li>
            <li>
              <strong>Root TAO stakers.</strong> Your share of every subnet&apos;s emission is
              pinned to its baseline at every change the owner makes, and it cannot be moved by
              sequencing changes. The global <code>TaoWeight</code> remains the governance lever
              over root&apos;s weight.
            </li>
            <li>
              <strong>Subnet owners.</strong> One change per week per subnet; the first is free of
              the limit. Lease-created subnets cannot drop the owner slot below 18%. Every{' '}
              <code>btcli</code> write validates locally and shows the exact floor, so test with{' '}
              <code>--dry-run</code> first.
            </li>
            <li>
              <strong>Integrators.</strong> Read the vector through the runtime API or the SDK read;
              treat <code>SubnetOwnerCut</code> / <code>OwnerCutEnabled</code> as legacy views;
              expect <code>SubnetEmissionWeightsSet</code> events and <code>Deprecated</code> from
              the two retired extrinsics.
            </li>
          </ul>
        </section>

        <section className={styles.section}>
          <h2 className={styles.subtitle}>Release details</h2>
          <p>
            Runtime <code>spec_version</code> <strong>480</strong>, <code>transaction_version</code>{' '}
            unchanged. Delivered to mainnet as a sudo runtime upgrade from <code>main</code>.
            Migration <code>seed_subnet_emission_weights_v1</code>, idempotent. Reference pages:{' '}
            <DocLink href='/docs/guides/etao'>Etao release guide</DocLink> (the complete standalone
            note), <DocLink href='/docs/concepts/emission-weights'>Emission weights</DocLink> (the
            concept page),{' '}
            <DocLink href='/docs/tx/set-subnet-emission-weights'>
              <code>set-subnet-emission-weights</code>
            </DocLink>
            ,{' '}
            <DocLink href='/docs/query/subnet-emission-weights'>
              <code>subnet-emission-weights</code>
            </DocLink>
            ,{' '}
            <DocLink href='/docs/hyperparameters/validator-emission'>
              <code>validator_emission</code>
            </DocLink>
            , and the source in{' '}
            <DocLink href='/code/pallets/subtensor/src/subnets/emission_weights.rs'>
              <code>emission_weights.rs</code>
            </DocLink>
            .
          </p>
        </section>
      </FadeInWrapper>
    </Suspense>
  );
};

export default page;
