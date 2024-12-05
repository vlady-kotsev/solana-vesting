"use client";

import { Keypair, PublicKey } from "@solana/web3.js";
import { useMemo, useState } from "react";
import { ellipsify } from "../ui/ui-layout";
import { ExplorerLink } from "../cluster/cluster-ui";
import {
  useTokenvestingProgram,
  useVestingProgramAccount,
} from "./tokenvesting-data-access";
import { useWallet } from "@solana/wallet-adapter-react";

export function TokenvestingCreate() {
  const { createVestingAccount } = useTokenvestingProgram();
  const [companyName, setCompanyName] = useState("");
  const [mint, setMint] = useState("");
  const { publicKey } = useWallet();

  const isFormValid = companyName.length > 0 && mint.length > 0;

  const handleSubmit = () => {
    if (!isFormValid || !publicKey) {
      return;
    }
    createVestingAccount.mutateAsync({
      companyName,
      mint,
    });
  };

  if (!publicKey) {
    return <p>Connect your wallet to create a vesting account</p>;
  }
  return (
    <div>
      <input
        type="text"
        className="input input-bordered input-primary w-full"
        placeholder="Company name"
        value={companyName}
        onChange={(e) => setCompanyName(e.target.value)}
      />
      <input
        type="text"
        className="input input-bordered input-primary w-full"
        placeholder="Mint"
        value={mint}
        onChange={(e) => setMint(e.target.value)}
      />
      <button
        className="btn btn-xs lg:btn-md btn-primary"
        disabled={!isFormValid || !publicKey || createVestingAccount.isPending}
        onClick={handleSubmit}
      >
        Create new vesting account {createVestingAccount.isPending && "..."}
      </button>
    </div>
  );
}

export function TokenvestingList() {
  const { accounts, getProgramAccount } = useTokenvestingProgram();

  if (getProgramAccount.isLoading) {
    return <span className="loading loading-spinner loading-lg"></span>;
  }
  if (!getProgramAccount.data?.value) {
    return (
      <div className="alert alert-info flex justify-center">
        <span>
          Program account not found. Make sure you have deployed the program and
          are on the correct cluster.
        </span>
      </div>
    );
  }
  return (
    <div className={"space-y-6"}>
      {accounts.isLoading ? (
        <span className="loading loading-spinner loading-lg"></span>
      ) : accounts.data?.length ? (
        <div className="grid md:grid-cols-2 gap-4">
          {accounts.data?.map((account) => (
            <TokenvestingCard
              key={account.publicKey.toString()}
              account={account.publicKey}
            />
          ))}
        </div>
      ) : (
        <div className="text-center">
          <h2 className={"text-2xl"}>No accounts</h2>
          No accounts found. Create one above to get started.
        </div>
      )}
    </div>
  );
}

function TokenvestingCard({ account }: { account: PublicKey }) {
  const { accountQuery, createEmployeeVesting } = useVestingProgramAccount({
    account,
  });
  const [startTime, setStartTime] = useState(0);
  const [endTime, setEndTime] = useState(0);
  const [cliffTime, setCliffTime] = useState(0);
  const [totalAmount, setTotalAmount] = useState(0);
  const [beneficiary, setBeneficiary] = useState("");
  const companyName = useMemo(
    () => accountQuery.data?.companyName ?? "",
    [accountQuery.data?.companyName]
  );

  return accountQuery.isLoading ? (
    <span className="loading loading-spinner loading-lg"></span>
  ) : (
    <div className="card card-bordered border-base-300 border-4 text-neutral-content">
      <div className="card-body items-center text-center">
        <div className="space-y-6">
          <h2
            className="card-title justify-center text-3xl cursor-pointer"
            onClick={() => accountQuery.refetch()}
          >
            {companyName}
          </h2>
          <div className="space-y-4">
            <input
              type="number"
              className="input input-bordered input-primary w-full"
              value={startTime || ""}
              placeholder="Start time"
              onChange={(e) => setStartTime(parseInt(e.target.value))}
            />
            <input
              type="number"
              className="input input-bordered input-primary w-full"
              value={endTime || ""}
              placeholder="End time"
              onChange={(e) => setEndTime(parseInt(e.target.value))}
            />
            <input
              type="number"
              className="input input-bordered input-primary w-full"
              value={cliffTime || ""}
              placeholder="Cliff time"
              onChange={(e) => setCliffTime(parseInt(e.target.value))}
            />
            <input
              type="number"
              className="input input-bordered input-primary w-full"
              value={totalAmount || ""}
              placeholder="Total amount"
              onChange={(e) => setTotalAmount(parseInt(e.target.value))}
            />
            <input
              type="text"
              className="input input-bordered input-primary w-full"
              value={beneficiary}
              placeholder="Beneficiary"
              onChange={(e) => setBeneficiary(e.target.value)}
            />
          </div>
          <div className="card-actions justify-around">
            <button
              className="btn btn-xs lg:btn-md btn-outline"
              onClick={() =>
                createEmployeeVesting.mutateAsync({
                  startTime,
                  endTime,
                  cliffTime,
                  totalAmount,
                  beneficiary,
                })
              }
              disabled={createEmployeeVesting.isPending}
            >
              Create employee vesting account
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
