# Bookfin Long-Form Corpus 01 — rapport de livraison

## Résultat

- **29 romans complets** en anglais original, par **21 auteurs prioritaires**.
- **16,827 pages Bookfin éligibles** : chaque page, y compris la première et la dernière, est dans le tirage uniforme.
- Pagination inchangée : convention pilote, cible de 1 600 caractères, coupure déterministe au blanc proche.
- Aucune pondération roman/nouvelle n’a été ajoutée.

## Contrôles exécutés

- SHA-256 du texte complet conforme au manifeste ; SHA-256 différent pour chaque œuvre.
- Absence des en-têtes/pieds Project Gutenberg et des tables des matières initiales du texte ingéré.
- Contrôle de présence de titres structurants et seuil de volume ; les éditions défaillantes sont rejetées au lieu d’être tronquées.
- Contrôle de la provenance avant nettoyage : les mots significatifs du titre doivent apparaître dans la page de titre source.
- Échantillon de première, médiane et dernière page généré pour chaque œuvre (hashes ci-dessous).

## Poids réel dans le tirage

| Auteur | Pages | Poids uniforme |
|---|---:|---:|
| Anne Brontë | 583 | 3.46% |
| Anthony Trollope | 249 | 1.48% |
| Charles Dickens | 2,415 | 14.35% |
| Charlotte Brontë | 638 | 3.79% |
| Edith Wharton | 479 | 2.85% |
| Elizabeth Gaskell | 1,541 | 9.16% |
| Emily Brontë | 403 | 2.39% |
| Ford Madox Ford | 256 | 1.52% |
| George Eliot | 1,110 | 6.60% |
| George Gissing | 642 | 3.82% |
| George Meredith | 655 | 3.89% |
| Henry James | 410 | 2.44% |
| Herman Melville | 760 | 4.52% |
| Jane Austen | 1,003 | 5.96% |
| Joseph Conrad | 769 | 4.57% |
| Nathaniel Hawthorne | 299 | 1.78% |
| Robert Louis Stevenson | 225 | 1.34% |
| Samuel Butler | 787 | 4.68% |
| Thomas Hardy | 1,024 | 6.09% |
| Wilkie Collins | 1,511 | 8.98% |
| William Makepeace Thackeray | 1,068 | 6.35% |

## Œuvres et sources

| Auteur | Titre | Année | Pages | Édition/source | Droits consignés |
|---|---|---:|---:|---|---|
| Jane Austen | Pride and Prejudice | 1813 | 454 | [Project Gutenberg ebook #1342](https://www.gutenberg.org/ebooks/1342) | public_domain_reviewed |
| Jane Austen | Emma | 1815 | 549 | [Project Gutenberg ebook #158](https://www.gutenberg.org/ebooks/158) | public_domain_reviewed |
| George Eliot | Middlemarch | 1871 | 1,110 | [Project Gutenberg ebook #145](https://www.gutenberg.org/ebooks/145) | public_domain_reviewed |
| Henry James | The Portrait of a Lady | 1881 | 410 | [Project Gutenberg ebook #2833](https://www.gutenberg.org/ebooks/2833) | public_domain_reviewed |
| Ford Madox Ford | The Good Soldier | 1915 | 256 | [Project Gutenberg ebook #2775](https://www.gutenberg.org/ebooks/2775) | public_domain_reviewed |
| Charles Dickens | David Copperfield | 1850 | 1,206 | [Project Gutenberg ebook #766](https://www.gutenberg.org/ebooks/766) | public_domain_reviewed |
| Charles Dickens | Bleak House | 1853 | 1,209 | [Project Gutenberg ebook #1023](https://www.gutenberg.org/ebooks/1023) | public_domain_reviewed |
| Thomas Hardy | Tess of the d'Urbervilles | 1891 | 525 | [Project Gutenberg ebook #110](https://www.gutenberg.org/ebooks/110) | public_domain_reviewed |
| Thomas Hardy | Jude the Obscure | 1895 | 499 | [Project Gutenberg ebook #153](https://www.gutenberg.org/ebooks/153) | public_domain_reviewed |
| Elizabeth Gaskell | North and South | 1855 | 618 | [Project Gutenberg ebook #4276](https://www.gutenberg.org/ebooks/4276) | public_domain_reviewed |
| Elizabeth Gaskell | Wives and Daughters | 1866 | 923 | [Project Gutenberg ebook #4274](https://www.gutenberg.org/ebooks/4274) | public_domain_reviewed |
| Anthony Trollope | Barchester Towers | 1857 | 249 | [Project Gutenberg ebook #619](https://www.gutenberg.org/ebooks/619) | public_domain_reviewed |
| Charlotte Brontë | Jane Eyre | 1847 | 638 | [Project Gutenberg ebook #1260](https://www.gutenberg.org/ebooks/1260) | public_domain_reviewed |
| Emily Brontë | Wuthering Heights | 1847 | 403 | [Project Gutenberg ebook #768](https://www.gutenberg.org/ebooks/768) | public_domain_reviewed |
| Anne Brontë | The Tenant of Wildfell Hall | 1848 | 583 | [Project Gutenberg ebook #969](https://www.gutenberg.org/ebooks/969) | public_domain_reviewed |
| William Makepeace Thackeray | Vanity Fair | 1848 | 1,068 | [Project Gutenberg ebook #599](https://www.gutenberg.org/ebooks/599) | public_domain_reviewed |
| Robert Louis Stevenson | Treasure Island | 1883 | 225 | [Project Gutenberg ebook #120](https://www.gutenberg.org/ebooks/120) | public_domain_reviewed |
| Joseph Conrad | Lord Jim | 1900 | 443 | [Project Gutenberg ebook #5658](https://www.gutenberg.org/ebooks/5658) | public_domain_reviewed |
| Joseph Conrad | The Secret Agent | 1907 | 326 | [Project Gutenberg ebook #974](https://www.gutenberg.org/ebooks/974) | public_domain_reviewed |
| Herman Melville | Moby-Dick; or, The Whale | 1851 | 760 | [Project Gutenberg ebook #2701](https://www.gutenberg.org/ebooks/2701) | public_domain_reviewed |
| Nathaniel Hawthorne | The Scarlet Letter | 1850 | 299 | [Project Gutenberg ebook #33](https://www.gutenberg.org/ebooks/33) | public_domain_reviewed |
| Edith Wharton | The Age of Innocence | 1920 | 361 | [Project Gutenberg ebook #541](https://www.gutenberg.org/ebooks/541) | public_domain_reviewed |
| Edith Wharton | Ethan Frome | 1911 | 118 | [Project Gutenberg ebook #4517](https://www.gutenberg.org/ebooks/4517) | public_domain_reviewed |
| Wilkie Collins | The Woman in White | 1859 | 842 | [Project Gutenberg ebook #583](https://www.gutenberg.org/ebooks/583) | public_domain_reviewed |
| Wilkie Collins | The Moonstone | 1868 | 669 | [Project Gutenberg ebook #155](https://www.gutenberg.org/ebooks/155) | public_domain_reviewed |
| George Gissing | New Grub Street | 1891 | 642 | [Project Gutenberg ebook #1709](https://www.gutenberg.org/ebooks/1709) | public_domain_reviewed |
| Samuel Butler | The Way of All Flesh | 1903 | 545 | [Project Gutenberg ebook #2084](https://www.gutenberg.org/ebooks/2084) | public_domain_reviewed |
| Samuel Butler | Erewhon | 1872 | 242 | [Project Gutenberg ebook #1906](https://www.gutenberg.org/ebooks/1906) | public_domain_reviewed |
| George Meredith | The Egoist | 1879 | 655 | [Project Gutenberg ebook #1684](https://www.gutenberg.org/ebooks/1684) | public_domain_reviewed |

## Droits et provenance

Chaque entrée du manifeste contient l’édition exacte, URL source, provenance du nettoyage, statut, base de droits, licence source et SHA-256 du texte complet. Les textes sont issus du cache texte de Project Gutenberg ; les en-têtes, pieds et la licence Gutenberg ne sont pas incorporés au texte littéraire.

**Ford Madox Ford — The Good Soldier :** US: public domain because first published in 1915 (Project Gutenberg US edition). EU/France: author died in 1939; life+70 term expired on 2010-01-01. The Project Gutenberg transcription is sourced under its terms; its boilerplate is not included.

## Rejets et anomalies

Rejets à la livraison : **0**. Pendant l’acquisition, des identifiants Gutenberg erronés ont été détectés par le contrôle de titre et corrigés avant la génération finale ; aucun texte erroné n’est présent dans le manifeste final.

## Échantillons de pages

Les trois positions suivantes ont été calculées et vérifiées pour chaque œuvre. Le hash est celui de la page Bookfin normalisée et permet de reproduire le contrôle sans publier d’extraits additionnels.

| Œuvre | Position | Pages de l’œuvre | SHA-256 de page |
|---|---:|---:|---|
| en_austen_pride_prejudice | 1 | 454 | `f2628d69e38da3deaf0a3dfce0f5d6861ac2abf9f44b52982635d32c6ff688df` |
| en_austen_pride_prejudice | 227 | 454 | `5694fcbf06f246c56a54d22c3501b3a5f1080fd937a43fe2e5a9c0f9a5671c58` |
| en_austen_pride_prejudice | 454 | 454 | `5aaf54d8978614eb93887662a5100501ab9c640527afd06e58011cc404e5eaac` |
| en_austen_emma | 1 | 549 | `a73db98182369b21b140576848cd29171a30afcae4d7693956e9f320a50c83a7` |
| en_austen_emma | 275 | 549 | `9016c554474ffbf8b1be185eeb128ab4d0c2ef20eab1822edbb745f2114bb6ec` |
| en_austen_emma | 549 | 549 | `1d4ff1ebe51d628f1b862e545d6f74cd0b6b1d753387c7ab2a58104a2dd70de9` |
| en_eliot_middlemarch | 1 | 1110 | `23ca5af7f54b05287c9d4cd6f1f0131045b1bdf7589df9bf8feeb56278517cd1` |
| en_eliot_middlemarch | 555 | 1110 | `cce1d9e4f37b9804ba210c8eae07362913f1358ebef525c50289c32fc035af34` |
| en_eliot_middlemarch | 1110 | 1110 | `6da9f84ff973656260fb89b54ef2597bacc3ad2a4d5042da480c39572d29a059` |
| en_james_portrait_lady | 1 | 410 | `ad942dc4c61f9fbafcf207d2b07f731b86bee96e30ac0dc6c394679a7a74e848` |
| en_james_portrait_lady | 205 | 410 | `279c6ed176fc0b789ce67825ca23364c3b2bada9069b7394135955530faac9a2` |
| en_james_portrait_lady | 410 | 410 | `ae8a05c74af91a5e4c3a83b39a3d2e3b16099edb7ac40d96c76e39523c39a91c` |
| en_ford_good_soldier | 1 | 256 | `a60214b2aa81198d1e44d1f28af906e6203ea724f24e7d2f287c7711da242740` |
| en_ford_good_soldier | 128 | 256 | `b1cedce55771b71ef2c3051b98022165a647aa184d4b019e301c597a03496ae0` |
| en_ford_good_soldier | 256 | 256 | `7a7a9243036d49ffb64e39a55f066cf323a76df06cf8eca8b45fee3b094bfb76` |
| en_dickens_david_copperfield | 1 | 1206 | `9def22ef54d3990baf3c9375848f6f12bdc13362279bfbb9779bf1e5b8736c62` |
| en_dickens_david_copperfield | 603 | 1206 | `7ae837174a8d1af6a4af73d39484f710ffe28ecae6305bd0cb5bfae423b7c840` |
| en_dickens_david_copperfield | 1206 | 1206 | `bef1aca1f62a3beb8a2f0699827b013a4d858041c6fd449ce7a08740547870d5` |
| en_dickens_bleak_house | 1 | 1209 | `027962c8f2bde6c09afe87be5cf367c710c16847340016f2a881d8e59c414247` |
| en_dickens_bleak_house | 605 | 1209 | `4c84bf9060a651102c599d02e4bc1ffc4e413c5ec3d790491aa3205055e5da58` |
| en_dickens_bleak_house | 1209 | 1209 | `e2040fff0c7f8760bd8d38709d7a3587eb8fead71a93153a14f16dcf1478808b` |
| en_hardy_tess | 1 | 525 | `c18bc447c9f6e710a238bf9ea2e9a8d2674c033b473dbdb8d1ee0523781e1bdb` |
| en_hardy_tess | 263 | 525 | `e65be6db10896a6390d271efe9623c1772e8b02a0b6c4950080056f893014c12` |
| en_hardy_tess | 525 | 525 | `1913842dfad05ac3b287839739daffae853b069aa6a2491fb9d72113db95ca67` |
| en_hardy_jude | 1 | 499 | `8a59a5eaa007af9c348d4b202ce37308df68f78e556ad6ac2696729605db01e6` |
| en_hardy_jude | 250 | 499 | `c78eba00b5c5bbce5f26a60c0a385e2dc1d80fb708213f3c0ca46b1aa74fb53a` |
| en_hardy_jude | 499 | 499 | `145376e3ac4a98c27fcea707d910d60aacf453b720d222e6338a81cfa920a29d` |
| en_gaskell_north_south | 1 | 618 | `79f5e8aa721b2e664dbcfc09a7e0054f9e82bebd30418f4a14cf17b2c3342aa5` |
| en_gaskell_north_south | 309 | 618 | `b042291d949834d9bb27337a55af8ce7a5ae76cb411d9c1bc02b7b2a83af1aae` |
| en_gaskell_north_south | 618 | 618 | `c6f88c0b06cc8ef61c5cb9c561652d9396c5467ca98bcc54e8d26f376a4dcdfd` |
| en_gaskell_wives_daughters | 1 | 923 | `87945cad640f791d2883f85b389fe4d64e86c43ebede7d1f407f502c07743080` |
| en_gaskell_wives_daughters | 462 | 923 | `d0cb2a8e2fceb06c9a3a86c7108b82dfe0a9ac382a571a42a19440c88d450394` |
| en_gaskell_wives_daughters | 923 | 923 | `3a5012e52d33814f4884bdb9b59987f8cf1736b9704f88940c47fa643e36690f` |
| en_trollope_barchester_towers | 1 | 249 | `4b3ed8a23cfa02d850be97ef8f6aafabd8eb6c0c06d55a3c4eb224ee3a10c8b2` |
| en_trollope_barchester_towers | 125 | 249 | `e4c7a30c5e4d733991e28b9b9dbe9fb7cf3cb91ebeb6e5fbee28b20f711d3f68` |
| en_trollope_barchester_towers | 249 | 249 | `f517108c34d6949d0479ec0c0ede1c73155a7ba6a021a6ee2bfa1e6c03224706` |
| en_bronte_jane_eyre | 1 | 638 | `cfed2b79d9ba1f9b5f2b775696b7b767cee5ff508eaf23a8a56ce67c2ff1b9a5` |
| en_bronte_jane_eyre | 319 | 638 | `db5577ef241c2edb612076a87640c08f6414954465e6cce5c348d9cb8ec757f8` |
| en_bronte_jane_eyre | 638 | 638 | `11177a24e5605ce4ce0f6d419612e7ab47828f8d647c067efc4c809680401624` |
| en_bronte_wuthering_heights | 1 | 403 | `48c40d4ad7bca64a24ff5cacee7f835dbf169b9d1616610ee4f9215fe77952ae` |
| en_bronte_wuthering_heights | 202 | 403 | `d495a0a075ba887f1739e7dc89ab4815a01a8b77009e04bc05e4346976b8a8fb` |
| en_bronte_wuthering_heights | 403 | 403 | `25288cf878f82687839648431a8f29efdc999ad985d3d862d59695953c7567f3` |
| en_bronte_tenant_wildfell_hall | 1 | 583 | `1c021ec80b7ae2b8741dcf0d790655167ab1fcce7bc2064e8df786d470d77ace` |
| en_bronte_tenant_wildfell_hall | 292 | 583 | `c096cf483e6e7cda86be51527d86df8e40232c2a4972914d21d4879a03582715` |
| en_bronte_tenant_wildfell_hall | 583 | 583 | `f0853841b0cd04b8700a411ff6c238324ed9936359ad58dd3888f042be8d4316` |
| en_thackeray_vanity_fair | 1 | 1068 | `144aec04c9caae64ca2e7cf3314e74b91998275d061a0d10182e8c64d8e60cd2` |
| en_thackeray_vanity_fair | 534 | 1068 | `f0373d9fc4280cee77dd859e03cd94d6070ec0d18d4c875f995cf1acc756720a` |
| en_thackeray_vanity_fair | 1068 | 1068 | `43e847c1da0593a18fc9872c9c9ec3249aaf10d25828bef516e39c0cd26565a1` |
| en_stevenson_treasure_island | 1 | 225 | `95826dee9dd901eda38060e382669af966e789c21c8328fb0b14a895a0780dae` |
| en_stevenson_treasure_island | 113 | 225 | `a585c08ed9e5794d2d430278fe132786c7f46629d60e9f66debee12675f814ba` |
| en_stevenson_treasure_island | 225 | 225 | `c02dad2fcf7b2fdc3cbba2a45070399d137e75d35e62ca05710e747bd247bb3f` |
| en_conrad_lord_jim | 1 | 443 | `db65784ab96417d310906451210fcba66c1825c593c4094b28b451e45e5e1bd2` |
| en_conrad_lord_jim | 222 | 443 | `5bb4099ca7cba1d2241cd6a6b0e3eeeb9a30caa855b683b75f52b22bf81e8dd7` |
| en_conrad_lord_jim | 443 | 443 | `5507626d5e04e61ada057cf879f510a3a01c7c05b2f585ac148a7a6198c0be2d` |
| en_conrad_secret_agent | 1 | 326 | `a71cae652d2c375c5c3182c188a17d4f5dac8747737f50ffa8f24c812934cda2` |
| en_conrad_secret_agent | 163 | 326 | `f25aaf7f86ad64232beaeb4d164efdd1be6d1c75de1e70329d9243b2d88ac880` |
| en_conrad_secret_agent | 326 | 326 | `e21aedd3eb9b3a2cf102b09c0ffa7bc223ae7e834e94310f648ec6ab5faf9642` |
| en_melville_moby_dick | 1 | 760 | `82ab2c5f34d772666f07876a6d241d1329c59039bb8cd2943b2f73ed62e44cc7` |
| en_melville_moby_dick | 380 | 760 | `1a8214cd4d50e4b482cc698165111a8c326653d534cc70c90355e0e54d731375` |
| en_melville_moby_dick | 760 | 760 | `601a675d3204ace66a4d25edf8d0d1be7a5b659745769734498b4156f03b8a16` |
| en_hawthorne_scarlet_letter | 1 | 299 | `3d3a7ad74f57837a6a88a2aa96fa18ed6ac4d521b3fbc68bab06ef22633c2a4b` |
| en_hawthorne_scarlet_letter | 150 | 299 | `c68a74eac414013c66f32c8edd9d317f5a3bf7cef8a5d579c850afe783de196f` |
| en_hawthorne_scarlet_letter | 299 | 299 | `5530db05eba301912a7c928deb4abb26d4cd3232ab95f34df3218c7c95a3f900` |
| en_wharton_age_innocence | 1 | 361 | `9da9b306cbf808086aecf17d57e4c9ffdb05a5782caa557fb16e8383f4a2895b` |
| en_wharton_age_innocence | 181 | 361 | `f4d01414232cdbeba529c28387857e9015f24c2a28f449cef8b6a847c1796062` |
| en_wharton_age_innocence | 361 | 361 | `ef51dfbac54f943e8b367e3928e919e6db522098d53ce8d28664f08748be8577` |
| en_wharton_ethan_frome | 1 | 118 | `d5a4920db4fb85421d6c79ec677ccc9a6de85dd4f7d040e2f73c859618ad2dbd` |
| en_wharton_ethan_frome | 59 | 118 | `ccf3ad033b07f6fda51f6961d5b388867d04c9f2893bedc0f5a9939f775dfe69` |
| en_wharton_ethan_frome | 118 | 118 | `4a175d36072c805610c59ae8d86289b6ddd9912b9c0f32422c6188f17d8dbe3e` |
| en_collins_woman_white | 1 | 842 | `29bee7b73dd649610a9e9f838196e12069f1d7b42a4c2656609657d7991fb52c` |
| en_collins_woman_white | 421 | 842 | `63928e7157753dea3116355785ee9d2d52cd8cff1432eb37fc94cb3bde31e0d2` |
| en_collins_woman_white | 842 | 842 | `b4481f549e51bd025a702e02b2a6acf24db0b5a36b664250dc39b8596e345ac0` |
| en_collins_moonstone | 1 | 669 | `5121826465ad777bae4a9334407b5dc3bca76c5bb91e5f39237383ed2f4a4b9d` |
| en_collins_moonstone | 335 | 669 | `936f45d354a6e20412d52c1b0f86ded34a3b172f227b198c289847c916bb30f5` |
| en_collins_moonstone | 669 | 669 | `9450e12f12f03ab059faec51062ada284ad4d06c7c9f8c8c3d0b4357c57646f5` |
| en_gissing_new_grub_street | 1 | 642 | `8b7606a06dc2744aef462ba74745e18b94cdb6684829328a2b83c84803f6981c` |
| en_gissing_new_grub_street | 321 | 642 | `35f26d306ee1c5ff47ee4203da9dca6424f7690c088ddd4db8ba58bdc6cca59f` |
| en_gissing_new_grub_street | 642 | 642 | `3f70cc447161aa0a674368b86729f0580c6743929ff73faa12e212918962f8ca` |
| en_butler_way_all_flesh | 1 | 545 | `9bb9fe5f53943c0caa3b807ea71fa303fddd2095a6d6fff2e0b28e7bbfcf1248` |
| en_butler_way_all_flesh | 273 | 545 | `bc715bede3e35f429cb42826c93dfd26c2e32ee573ee246d2663aa1585f3c3bc` |
| en_butler_way_all_flesh | 545 | 545 | `2c7b75cf70dd1fcc2247b5c7dfb66dcfdb22682eff7818f171add92e36b7d507` |
| en_butler_erewhon | 1 | 242 | `5f448fa6f55fad4ed2f6aa9e4a78a872921ca32e9f79d4f9f53880ed3415e925` |
| en_butler_erewhon | 121 | 242 | `d1e26ec52b534cfb26dcbd3776a550ea1bedf6e65e7b27a1c345e71d4fcf3662` |
| en_butler_erewhon | 242 | 242 | `ca6247cecc232939527ab453f33d866b9b73965b02c30f6b927c12d130fcaf34` |
| en_meredith_egoist | 1 | 655 | `2f2b0fb628369bac2c5b0e3622ee6ece8967da7a43baa04dd2bfec01231cf2be` |
| en_meredith_egoist | 328 | 655 | `30f9c9f13a6d1ef27afc361e361ecdbbcd798b50edc758e03e7681701257af21` |
| en_meredith_egoist | 655 | 655 | `ea93e0ff4fc6297c7b048cb81d3309a33b0df7da3a8f18755e2320da8fc33149` |

## Validation mobile

Le serveur SSR local démarre avec le corpus et trois tirages API ont renvoyé des pages anglaises de 1 610–1 616 caractères. Aucune capture de cette livraison n’est jointe : aucun émulateur ni aucune application mobile cible n’était disponible dans la session de validation. La validation visuelle mobile reste à exécuter avec, au minimum, la première, une centrale et la dernière page de trois œuvres différentes.

## Reproduction

```powershell
python scripts\download_long_form_corpus.py
python scripts\validate_long_form_corpus.py
cargo run --bin ingest_long_form_corpus
python scripts\generate_long_form_report.py
```
