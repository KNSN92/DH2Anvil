# DH2Anvil
DistantHorizons modのLODデータをanvilフォーマットへ復元するツールです。

ワールドデータを無くしたけど、DistantHorizons modのLODデータは残ってた！
という時に助かると思います。実際その為に作ったし。

## どうやって使うの？

### データベース情報の確認
データベースの情報を確認できます：
```bash
dh2anvil info DistantHorizons.sqlite
```

### データ変換
LODデータをAnvilフォーマットに変換します：
```bash
# 基本的な変換
dh2anvil convert DistantHorizons.sqlite

# 出力先フォルダを指定
dh2anvil convert DistantHorizons.sqlite --out ./my_region

# 範囲を指定して変換（リージョン座標 ±5 の範囲のみ）
dh2anvil convert DistantHorizons.sqlite --range 5

# スレッド数を指定
dh2anvil convert DistantHorizons.sqlite --threads 8

# 既存ファイルを上書き
dh2anvil convert DistantHorizons.sqlite --overwrite
```