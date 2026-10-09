# DAGLight in pseudocode

The colouring engine of `protocol/`, one procedure per step, with the names the code uses. Every
quantity a block derives depends on its past alone; nothing about consensus is transmitted.

## Parameters and notation

```
bps                 blocks per second; interval = 1000 / bps ms
F                   finality horizon in ms (12 h)
H                   handshake, in delays (4): the margin that decides a fork, in delays of work,
                    and the longest an honest block is assumed to be on its way

B                   a block: id, selected parent S, other parents, work w(B), stamp t(B)
chain(B)            B's selected chain from genesis; chain(B)[k] its block at height k
past(B)             everything reachable from B, B included
M                   B's mergeset: ⋃ past(p) over the other parents p, minus past(S); what the
                    other parents add to S's past, B itself not included

P(B)                what every node derives for B:
  past_work         Σ w over past(B)
  chain_work        Σ w over chain(B)
  final_height      highest k with t(chain(B)[k]) ≤ t(B) − F            (inherited, only rises)
  τ                 folding threshold: lowest height whose fork is still open
  folded            contested work at the forks closed for good below τ
  forks[k]          for each open fork at height k ≥ τ:
    rival[c]        for each rival child c of chain[k]: (contested, settled, chain)
    held            the most contested work the lineage has ever had foldable here
  N                 what the lineage knows of the network (see Learn):
                    block_work, rate, delay, delay_work, width E

final_point(B)   = chain(B)[final_height(B)]               # below it a node never reorganises
pruning_point(B)  = final_point(final_point(B))             # below it nothing is ever read again

blue(B)   = weigh(N(S), chain_work, chain_work + folded + Σ_k forks[k].held)    # a tip's weight
      # its cone of credited work, weighed as a side is: entangled in full, a lone chain at 1/E
P(a) > P(b)  iff  blue(a) > blue(b), or equal and id(a) < id(b)           # ties to the lower hash
margin(N) = H · N.delay_work
```

## Validity of B, given S and the other parents

```
valid(B):
  all parents known
  w(B) == N(S).block_work                                   # the difficulty S's lineage requires
  for each other parent p:
      not P(p) > P(S)                                       # B extends the heaviest parent it lists
      final_point(S) ∈ chain(p)                             # p agrees with S on finality
  t(B) ≥ t(p) for every parent p                            # stamped no earlier than any parent
  for each x ∈ M:  join(x) ≥ pruning_height(S)             # nothing merged forks off below pruning
```

A node also parks a block stamped beyond its clock until the clock reaches it, and waits for
missing parents, so blocks enter in causal order and stamps buy no weight.

## Deriving P(B)

```
derive(B):
  S = selected parent;  P = copy of P(S)
  for x ∈ M:                                                # the mergeset: what the other parents
                                                            # add to past(S); read off the topology
      join(x)    = height where chain(x) meets chain(S)
      rival(x)   = chain(x)[join(x) + 1]                    # the fork block's child that x backs
      depth(x)   = chain_work(x) − chain_work(chain(S)[join(x)])
      missed(x)  = past_work(S) − work(past(S) ∩ past(x))   # what x had not seen of S's past
      settled(x) = τ(x) > join(x)                           # x's lineage already closed this fork
  credited = { x ∈ M : join(x) ≥ τ(S) }                     # blue; the rest are red
  seen = past_work(S) + Σ_{x∈M} w(x)                        # all work in B's past but B's own

  for x ∈ credited: Record(P, x)
  DropBelow(P, pruning_height(S))
  Decide(P, B);  Expire(P, B);  Judge(P, B)
  P.N = Learn(P, B)
  P.past_work  = seen + w(B)
  P.chain_work = chain_work(S) + w(B)
  P.final_height = max(P(S).final_height, highest k ≤ height(S) with t(chain(S)[k]) ≤ t(B) − F)
  return P
```

Red blocks are in `past_work` and in `seen`, feed the difficulty, and are sequenced; they are never
recorded, held, folded or sampled. Everything below is measured by `N(S)`, the parent's network.

### Record

```
Record(P, x):
  r = P.forks[join(x)].rival[rival(x)]                      # opening the fork if needed
  r.chain = max(r.chain, depth(x))
  if settled(x): r.settled += w(x)  else: r.contested += w(x)
```

### Weighing and the verdict at one fork

```
weigh(N, chain, cone)  = min(chain, cone / N.E) + (cone − chain)
      # the work beside a chain counts in full; the chain counts in full only once the cone is
      # as wide as the network; a lone chain weighs 1/E of its work

side(P, k, above)      = weigh(N, c, c + above),  c = chain_work(S) − chain_work(chain(S)[k])
      # S's chain above the fork, S included, B excluded, plus the rival work recorded at the forks
      # above k, whose lineages left the chain higher up and so vote with it here

weighed(r)             = weigh(N, min(r.chain, r.contested + r.settled), r.contested + r.settled)

led(P, k, c, r, side)  = weighed(r) < side
                         or (weighed(r) == side and k < height(S) and id(chain(S)[k+1]) < id(c))
      # a tie goes to the lower hash of the chain's continuation; at the parent's own height the
      # continuation is B itself, with no work yet, and the tie is lost
```

B's own work is on no side: a block observes the vote and never tips it.

### Decide: close forks for good

```
Decide(P, B):
  k = max(P.τ, pruning_height(S))
  while decided(P, B, k):
      P.folded += Σ_c P.forks[k].rival[c].contested        # settled work goes with the record
      remove P.forks[k]
      k += 1
  P.τ = k

decided(P, B, k):
  above    = Σ over open forks j ≠ k of (contested + settled at j)
  rivals   = Σ_c weighed(P.forks[k].rival[c])               # 0 if no fork is open at k
  return side(P, k, above) > rivals + margin(N)
```

At `k = height(S)` the side is empty, so deciding stops there at the latest.

### Expire: end a stalemate

```
Expire(P, B):
  f = P.τ
  while f < height(S) and t(chain(S)[f]) + F ≤ t(B):        # the fork block is a horizon old
      if P.forks[f] open:
          above = Σ over open forks j ≠ f of (contested + settled at j)
          if Σ_c settled > 0 or not all c: led(P, f, c, rival[c], side(P, f, above)): break
          P.folded += Σ_c contested;  remove P.forks[f]      # closed as judged
      f += 1
  P.τ = f
```

A fork the side does not lead outright stays open: the lineage has lost it.

### Judge: hold what the side leads

```
Judge(P, B):
  above = 0
  for each open fork k, highest first:
      s = side(P, k, above)
      foldable = Σ_c contested over rivals c with led(P, k, c, rival[c], s)
      P.forks[k].held = max(P.forks[k].held, foldable)      # a heavier rival never takes it back
      above += Σ_c (contested + settled at k)
```

### DropBelow: forget what can never matter again

```
DropBelow(P, h):
  for each open fork k < h:  P.folded += P.forks[k].held;  remove P.forks[k]
```

Folding converts `held` into `folded`, so `folded + Σ held` never falls along a chain, and nothing a
block lists takes back what its lineage had. `blue(B) > blue(S)` whenever `E` holds still; only a
change of `E`, which every child of the lineage inherits alike, can weigh a child below its parent.

## Learn: what the network teaches

```
Learn(P, B):
  N = copy of N(S)
  for x ∈ credited, by id: Sample(N, missed(x), w(x))       # one order, however they arrived
  Sample(N, 0, w(B))                                        # B itself missed nothing
  tip_work.add(past_work(B) − past_work(S), t(B) − t(S))    # a horizon sum: decays by elapsed/F
  N.block_work = tip_work / (F / interval)                  # difficulty: the tip's recent work
  fed = max(fed, pruning_height(S));  τ' = min(P.τ, height(S))
  if τ' > fed:                                              # a chain segment was newly folded
      segment = chain(S)[fed .. τ']
      folded_past.add(Δpast_work, Δt);  folded_chain.add(Δchain_work, Δt);  fed = τ'
      N.rate = folded_past / (F / interval)
      N.E    = max(1, folded_past / folded_chain)           # blocks per chain step, decided history
      N.delay_work = N.rate · N.delay
  else:
      N.delay_work = max(N.delay_work, N.rate · N.delay)    # rises at once, falls only with a fold
  return N

Sample(N, missed, work):                                    # a running median of the delay, in work
  if missed > margin(N): return                             # withheld, not delayed: no sample
  step = N.delay · min(work, N.delay_work) / (F / interval · N.block_work)
  if missed / N.block_work > N.delay: N.delay += step  else: N.delay = max(1, N.delay − step)
```

The pace and the width come only from the folded chain, decided history every lineage agrees on,
so a minority cannot teach the network a lower bar. The genesis seeds `block_work`, `delay` and
`E`; the DAG replaces them over horizons.

### Fixed network

`N` is an interface, and the rules above read it the same way whatever stands behind it. The
learned network is the default; the other implementation, `FixedNetworkPerception`, keeps the
genesis's `block_work`, `delay` and `E` for good, with `delay_work = block_work · delay / interval`,
and its `Sample` and `Learn` do nothing. That
separates the colouring from its estimators: the structural rules can be studied with every
yardstick held still, by `--fixed` in the simulator and `Config::fixed()` in the measured table,
whose `fixed*` rows pair with their learned twins.

## Tips, the next block, the order

```
tips       = blocks without children whose chain contains the node's finality point
heaviest   = max tip by P                                  # blue work, ties to the lower hash
final_point(node) = final_point(heaviest), never lower than before; tips missing it die

next_block = (selected parent = heaviest,
              other parents  = every other tip p with not P(p) > P(heaviest) and
                               final_point(heaviest) ∈ chain(p), dropping any through which the
                               block would merge something that joins below the pruning point)
              # no cap on the mergeset, no merge depth

order: for each chain block C with selected parent S, in chain order:
       blues of M(C) (join ≥ τ(S)) by (past_work, id), then reds by (past_work, id), then C
       a node reports a reorg whenever its heaviest tip changes sides, then the new side's sequence

```

## What follows

- `derive` reads `P(S)` and the mergeset; it costs one pass over `M` and the open forks. The order
  of arrival changes nothing.
- Honest blocks are blue: a fork closes only on a lead of `H` delays of work, and an honest block
  votes within one delay of the tips.
- The mirror rule: once a lineage folds a fork, blocks from the other side join below its `τ` and
  are red to it for good; its own blocks are `settled` to the other side, recorded as evidence but
  never held. From the first fold, neither side imports the other again.
- A lone chain weighs `1/E` of its work until it entangles, as a rival and as a tip alike, so a
  spine can neither lead a wide honest branch nor outrank it by luck; a race is won or lost whole,
  since the leader holds all of the trailer's contested work.
- A lineage decides from what it has seen alone. A minority partition folds its forks its own
  way; once the majority folds them the other way, the mirror rule keeps the minority's work out
  for good, whatever it decided. A stall of a whole horizon is ended by `Expire`.
