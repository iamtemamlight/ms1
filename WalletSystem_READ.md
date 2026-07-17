Your plan is solid. Since `manualwalletmodal.tsx` already exists, `WalletSystem.tsx` should focus on state management, table rendering, sorting, and invoking the modal.

Here's a clean architecture that keeps the component maintainable.

### Component Structure

```text
WalletSystem.tsx
│
├── Header
│     └── Add Wallet Button
│
├── Table 1 - Wallet List
│     ├── Sortable Header
│     ├── Sample Wallet Rows (6)
│     └── Total Row
│
├── Table 2 - Profit Withdrawal
│     ├── Auto / Manual Toggle
│     ├── Auto Threshold Input
│     └── Manual Amount Input
│
├── Table 3 - Transfer History
│     ├── Sortable Header
│     ├── History Rows
│     └── Total Row
│
└── ManualWalletModal
```

---

## State

```tsx
const [wallets, setWallets] = useState<Wallet[]>(sampleWallets);

const [history, setHistory] = useState(sampleTransfers);

const [showModal, setShowModal] = useState(false);

const [withdrawMode, setWithdrawMode] =
useState<"auto" | "manual">("auto");

const [autoThreshold, setAutoThreshold] =
useState(100);

const [manualAmount, setManualAmount] =
useState("");

const [walletSort, setWalletSort] = useState({
    key: "id",
    direction: "asc"
});

const [historySort, setHistorySort] = useState({
    key: "date",
    direction: "desc"
});
```

---

## Wallet Table Columns

| No | ID | Wallet Address | Private Key | Chain | Balance | Verify | Status | Actions |
| -- | -- | -------------- | ----------- | ----- | ------- | ------ | ------ | ------- |

Actions:

```
✏ Edit
🗑 Delete
💾 Save
```

Private Key column

```
**************AB4F
```

Status

```
🟢 Active

or

⚪ Disabled
```

Verify

```
✔ Verified

or

Verify Button
```

---

## Sample Wallets

```tsx
[
{
id:"W001",
address:"0x12...91ab",
privateKey:"***********A3F2",
chain:"EVM",
balance:12.34,
verified:true,
active:true
},
{
id:"W002",
address:"0x98...12bc",
privateKey:"***********CC92",
chain:"BSC",
balance:7.89,
verified:true,
active:true
},
{
id:"W003",
address:"0x76...23de",
privateKey:"***********D9A8",
chain:"SVM",
balance:0.56,
verified:false,
active:false
},
...
]
```

Create six entries.

---

## Total Row

```text
Total Wallets : 6

Combined Balance : 87.54
```

---

# Add Wallet Button

```tsx
<Button
onClick={()=>{
    // detect wallet
    // then open modal
    setShowModal(true);
}}
>
+ Add Wallet
</Button>
```

Modal

```tsx
<ManualWalletModal

open={showModal}

onClose={()=>setShowModal(false)}

onSave={handleWalletSave}

/>
```

---

## Profit Withdrawal Table

```
+----------------------------+
| Profit Withdrawal          |
+----------------------------+

Mode

(o) Auto

( ) Manual

If Auto

Minimum Withdrawal

[100 USDT]

If Manual

Amount

[________]
```

Logic

```tsx
{withdrawMode==="auto" ? (

<Input ... />

):(

<Input ... />

)}
```

---

## Transfer History

Columns

| No | Date | Amount | Wallet |

Sample

```
1

2025-06-12

150 USDT

0x12...91ab
```

Five or six rows.

Footer

```
Total Withdrawn

1,542 USDT
```

---

## Generic Sort Function

```tsx
const sortData = (
data,
key,
direction
) => {

return [...data].sort((a,b)=>{

if(a[key] < b[key])
return direction==="asc" ? -1 : 1;

if(a[key] > b[key])
return direction==="asc" ? 1 : -1;

return 0;

});

};
```

Header click

```tsx
<th
onClick={()=>handleWalletSort("balance")}
>

Balance ↑↓

</th>
```

---

## Styling

Since your project already has a modern dashboard aesthetic, keep the tables consistent:

* Rounded cards (`rounded-xl`)
* Subtle shadow (`shadow-md`)
* Zebra rows (`even:bg-gray-50`)
* Sticky headers
* Hover highlighting
* Green badges for verified/active
* Gray badges for inactive
* Responsive horizontal scrolling (`overflow-x-auto`)
* Sort indicators (`▲` / `▼`)

---

## Functional Requirements Checklist

* ✅ Three separate card sections
* ✅ Wallet List table
* ✅ Wallet detection → opens existing `ManualWalletModal`
* ✅ Six sample wallet rows
* ✅ Total wallet summary row
* ✅ Sortable wallet table
* ✅ Masked private keys
* ✅ Verify status/action
* ✅ Active/inactive toggle
* ✅ Edit/Delete/Save actions
* ✅ Profit Withdrawal section with Auto/Manual modes
* ✅ Configurable threshold or manual amount
* ✅ Transfer History table
* ✅ Sortable transfer history
* ✅ Total withdrawn row
* ✅ Fully responsive layout
* ✅ Ready to connect to backend APIs later without significant refactoring





analyse and  describe me th engine control page of allbrigh ton the dashboard: 1. simulation wizard;  and completion  2.  pilo tmode wizard 3. live mode wizard :   and the completion of each mode mus tbe completed by auto generating and auto archiving the report on the repor tpage of the sidebar ( dashboard ) 