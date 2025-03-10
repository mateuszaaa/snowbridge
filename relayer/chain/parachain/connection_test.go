// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain_test

import (
	"context"
	"testing"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/snowbridge/relayer/chain/parachain"
	"github.com/snowfork/snowbridge/relayer/crypto/sr25519"
)

func TestConnect(t *testing.T) {
	log := logrus.NewEntry(logrus.New())

	conn := parachain.NewConnection("ws://127.0.0.1:11144/", sr25519.Alice().AsKeyringPair(), log)
	err := conn.Connect(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	conn.Close()
}
