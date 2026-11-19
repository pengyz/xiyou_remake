#!/usr/bin/env python3
"""【keydemand 第二实现复核】独立复算每层「入场 → 任一上行出口」的最少入场钥匙向量。

与 Rust 实现 `crates/game-solver/src/keydemand.rs` **不共享代码**：本脚本自己解析
maplv/sprite、自己读 `data/tables/sprite_defs.json` 的 render_category/floor_category_override、
自己实现 BFS + 125 网格 + Pareto。用途 = 内部一致性判定（AGENTS.md §1「多路径必须给同一结果」）。

用法：
    cargo run -q -p game-solver --bin solver -- keydemand --out /tmp/keydemand.json
    python3 tools/keydemand_xcheck.py /tmp/keydemand.json      # 期望 `54/54 一致, mismatch=0`

口径（与 Rust 侧逐字同源，证据锚见 docs/findings/2026-11-20-key-demand.md）：
- 地形：maplv 头 u16LE×2（`>>1` = 真实宽高），网格 1 行距 `w*4`、列距 2，
  `walk = tile_walkability[tile] == 1`（复刻 solve_local.rs:1096-1108）。
- 免地形终审：ty 6/7/8（`stair_count`）；硬阻挡：ty 9/10/11/12、cat16（77/78）、cat32（72/84/87）。
- 条件门 ty83 与封印门 ty81/4 按可通行（假设 2）。
- 钥匙：ty26 黄 / ty27 红 ? -> 以 sprite_defs 的 cat2 与实体码 26/27/28 判定，色序 (y,b,r)。
"""
import json,struct,collections,sys,os
R=os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXTRA={7:2,8:2,76:2,82:2,4:1,5:1,9:1,57:1,59:1,70:1,71:1,72:1,73:1,77:1,78:1,81:1,83:1}
def sprites(f):
    d=open(f'{R}/assets/raw/sprite{f}','rb').read()
    n=struct.unpack_from('<I',d,0)[0]; o=4; out=[]
    for _ in range(n):
        ty=d[o]; x=struct.unpack_from('<H',d,o+1)[0]; y=struct.unpack_from('<H',d,o+3)[0]
        out.append((ty,x>>5,y>>5))
        o+=5+EXTRA.get(ty,0)
    return out
def maplv(f):
    d=open(f'{R}/assets/raw/maplv{f}','rb').read()
    rw,rh=struct.unpack_from('<HH',d,0); w,h=rw>>1,rh>>1
    return w,h,d[4:]
TW=[int(l) for l in open(f'{R}/crates/game-core/tests/fixtures/tile_walkability.txt').read().split()]
SD=json.load(open(f'{R}/data/tables/sprite_defs.json'))
CAT={e['type_id']:e['render_category'] for e in SD['sprite_defs']}
OVR={}
for e in SD['sprite_defs']:
    for o in (e.get('floor_category_override') or []):
        OVR[(o['floor'],e['type_id'])]=o['category']
def cat(ty,fl):
    return OVR.get((fl,ty),CAT.get(ty,0))
def analyze(fl,policy='open'):
    w,h,t=maplv(fl)
    ents=sprites(fl)
    passable=[False]*(w*h); blocked=[False]*(w*h); exempt=[False]*(w*h)
    doors=collections.defaultdict(list); picks=collections.defaultdict(list)
    door_col=[]; pick_col=[]; entry=None; goals=[]
    G={'gates':0,'mons':0,'haz':0,'hard':0}
    for ty,x,y in ents:
        if not(0<=x<w and 0<=y<h): continue
        i=y*w+x
        if ty==7 and i not in goals: goals.append(i)
        if ty==8 and entry is None: entry=i
        c=cat(ty,fl)
        if c==1:
            if ty in (1,2,3):
                s=len(door_col); door_col.append({1:0,3:1,2:2}[ty]); doors[i].append(s)
            elif ty in (4,81,83):
                G['gates']+=1
                if policy=='blocked': blocked[i]=True
            elif ty in (6,7,8):
                exempt[i]=True
                if ty==6: G['haz']+=1
            elif ty in (9,10,11,12):
                G['hard']+=1; blocked[i]=True
        elif c in (2,4):
            if ty in (26,27,28):
                s=len(pick_col); pick_col.append({26:0,28:1,27:2}[ty]); picks[i].append(s)
        elif c==8: G['mons']+=1
        elif c==16: G['hard']+=1; blocked[i]=True
        elif c==32: G['hard']+=1; blocked[i]=True
    for y in range(h):
        for x in range(w):
            i=y*w+x
            idx=y*w*4+x*2
            terr= t[idx]<len(TW) and TW[t[idx]]==1 if idx<len(t) else False
            passable[i]=(terr or exempt[i]) and not blocked[i]
    nd=len(door_col); npk=len(pick_col)
    pmask=[0,0,0]; dmask=[0,0,0]
    for s,c in enumerate(pick_col): pmask[c]|=1<<s
    for s,c in enumerate(door_col): dmask[c]|=1<<s
    pc=lambda m,c: bin(m&pmask[c]).count('1'); dc=lambda m,c: bin(m&dmask[c]).count('1')
    def search(keys, cap=200000):
        if entry is None: return None
        p0=0
        for s in picks[entry]: p0|=1<<s
        o0=0
        for d in doors[entry]:
            c=door_col[d]
            held=keys[c]+pc(p0,c)-dc(o0,c)
            if held<=0: return False
            o0|=1<<d
        if entry in goals: return True
        seen={(entry,p0,o0)}; q=collections.deque([(entry,p0,o0)])
        while q:
            pos,pk,op=q.popleft()
            x,y=pos%w,pos//w
            for dx,dy in ((0,-1),(0,1),(-1,0),(1,0)):
                nx,ny=x+dx,y+dy
                if not(0<=nx<w and 0<=ny<h): continue
                ni=ny*w+nx
                if not passable[ni]: continue
                no=op; denied=False
                for d in doors[ni]:
                    if no>>d&1: continue
                    c=door_col[d]
                    if keys[c]+pc(pk,c)-dc(no,c)<=0: denied=True; break
                    no|=1<<d
                if denied: continue
                npk=pk
                for s in picks[ni]: npk|=1<<s
                if ni in goals: return True
                st=(ni,npk,no)
                if st not in seen:
                    seen.add(st); q.append(st)
                    if len(seen)>cap: raise RuntimeError('cap')
        return False
    if len(goals)==0: return dict(status='no_up_exit',gates=G['gates'],mons=G['mons'],haz=G['haz'],hard=G['hard'],doors=nd,pick=npk)
    if entry is None: return dict(status='no_entry',gates=G['gates'],mons=G['mons'],haz=G['haz'],hard=G['hard'],doors=nd,pick=npk)
    if nd>24: return dict(status='too_many_doors',gates=G['gates'],mons=G['mons'],haz=G['haz'],hard=G['hard'],doors=nd,pick=npk)
    cands=sorted([[y,b,r] for y in range(5) for b in range(5) for r in range(5)], key=lambda v:(sum(v),v))
    feas=[]; infeas=[]
    def le(a,b): return all(a[i]<=b[i] for i in range(3))
    for v in cands:
        if any(le(f,v) for f in feas): continue
        if any(le(v,u) for u in infeas): continue
        (feas if search(v) else infeas).append(v)
    if feas:
        par=[v for v in feas if not any(f!=v and le(f,v) and not le(v,f) for f in feas)]
        par.sort(key=lambda v:(sum(v),v))
        return dict(status='ok',needed=par[0],pareto=par,gates=G['gates'],mons=G['mons'],haz=G['haz'],hard=G['hard'],doors=nd,pick=npk)
    st='needs_more_than_grid' if search([99,99,99]) else 'unreachable'
    return dict(status=st,gates=G['gates'],mons=G['mons'],haz=G['haz'],hard=G['hard'],doors=nd,pick=npk)

arg = sys.argv[1] if len(sys.argv)>1 else '/tmp/keydemand.json'
rust={}
for row in json.load(open(arg))['floors']:
    rust[row['floor']]=row
bad=0
for f in range(1,55):
    py=analyze(f); r=rust[f]
    rn=r['needed']; rn=[rn['y'],rn['b'],rn['r']] if rn else None
    pyp = [list(v) for v in py['pareto']] if py['status']=='ok' else []
    ok = py['status']==r['status'] and py.get('needed')==rn and pyp==[list(v) for v in r['pareto']] \
         and py['gates']==r['gates_ignored'] and py['mons']==r['monsters_ignored'] and py['haz']==r['hazards'] \
         and py['hard']==r['hard_blocks'] and py['doors']==r['doors'] and py['pick']==r['key_pickups']
    if not ok:
        bad+=1
        print(f'MISMATCH f{f}: py={py} rust_needed={rn} rust_status={r["status"]} rust_par={r["pareto"]} rust_counters=(g{r["gates_ignored"]},m{r["monsters_ignored"]},h{r["hazards"]},hard{r["hard_blocks"]},d{r["doors"]},p{r["key_pickups"]})')
print(f'== xcheck: {54-bad}/54 一致, mismatch={bad} ==')
