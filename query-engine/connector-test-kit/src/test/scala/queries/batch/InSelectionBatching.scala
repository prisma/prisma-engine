package queries.batch

import org.scalatest.{FlatSpec, Matchers}
import util.{ApiSpecBase, ProjectDsl}

class InSelectionBatching extends FlatSpec with Matchers with ApiSpecBase {
  val project = ProjectDsl.fromString {
    """model A {
      |  id Int @id
      |  b B
      |  c C
      |}
      |model B {
      |  id Int @id
      |  as A[]
      |}
      |model C {
      |  id Int @id
      |  as A[]
      |}
      |"""
  }

  override protected def beforeAll(): Unit = {
    super.beforeAll()
    database.setup(project)

    server.query(
      """mutation a {createA(data:{
        |  id: 1
        |  b: { create: { id: 1 } }
        |  c: { create: { id: 1 } }
        |}){id}}""",
      project = project
    )

    server.query(
      """mutation a {createA(data:{
        |  id: 2
        |  b: { connect: { id: 1 } }
        |  c: { create: { id: 2 } }
        |}){id}}""",
      project = project
    )

    server.query(
      """mutation a {createA(data:{
        |  id: 3
        |  b: { create: { id: 3 } }
        |  c: { create: { id: 3 } }
        |}){id}}""",
      project = project
    )

    server.query(
      """mutation a {createA(data:{
        |  id: 4
        |  b: { create: { id: 4 } }
        |  c: { create: { id: 4 } }
        |}){id}}""",
      project = project
    )

    server.query(
      """mutation a {createA(data:{
        |  id: 5
        |  b: { create: { id: 5 } }
        |  c: { create: { id: 5 } }
        |}){id}}""",
      project = project
    )
  }

  "batching of IN queries" should "work when having more than the specified amount of items" in {
    val res = server.query(
      """query idInTest {
        |   findManyA(where: { id_in: [5,4,3,2,1,1,1,2,3,4,5,6,7,6,5,4,3,2,1,2,3,4,5,6] }) { id }
        |}
        |""".stripMargin,
      project = project,
      legacy = false,
      batchSize = 2,
    )

    res.toString should be(
      """{"data":{"findManyA":[{"id":1},{"id":2},{"id":3},{"id":4},{"id":5}]}}""".stripMargin
    )
  }

  "ascending ordering of batched IN queries" should "work when having more than the specified amount of items" in {
    val res = server.query(
      """query idInTest {
        |   findManyA(where: { id_in: [5,4,3,2,1,2,1,1,3,4,5,6,7,6,5,4,3,2,1,2,3,4,5,6] }, orderBy: id_ASC) { id }
        |}
        |""".stripMargin,
      project = project,
      legacy = false,
      batchSize = 2,
    )

    res.toString should be(
      """{"data":{"findManyA":[{"id":1},{"id":2},{"id":3},{"id":4},{"id":5}]}}""".stripMargin
    )
  }

  "descending ordering of batched IN queries" should "work when having more than the specified amount of items" in {
    val res = server.query(
      """query idInTest {
        |   findManyA(where: {id_in: [5,4,3,2,1,1,1,2,3,4,5,6,7,6,5,4,3,2,1,2,3,4,5,6] }, orderBy: id_DESC) { id }
        |}
        |""".stripMargin,
      project = project,
      legacy = false,
      batchSize = 2,
    )

    res.toString should be(
      """{"data":{"findManyA":[{"id":5},{"id":4},{"id":3},{"id":2},{"id":1}]}}""".stripMargin
    )
  }

  "ascending ordering of batched IN with relation field" should "work" in {
    val res = server.query(
      """
        |query {
        |  findManyB {
        |		as(orderBy: c_ASC) {
        |      c { id }
        |    }
        |  }
        |}
        |""".stripMargin,
      project = project,
      legacy = false,
      batchSize = 2,
    )

    res.toString should be(
      """{"data":{"findManyB":[{"as":[{"c":{"id":1}},{"c":{"id":2}}]},{"as":[{"c":{"id":3}}]},{"as":[{"c":{"id":4}}]},{"as":[{"c":{"id":5}}]}]}}""".stripMargin
    )
  }

  "descending ordering of batched IN with relation field" should "work" in {
    val res = server.query(
      """
        |query {
        |  findManyB {
        |		as(orderBy: c_DESC) {
        |      c { id }
        |    }
        |  }
        |}
        |""".stripMargin,
      project = project,
      legacy = false,
      batchSize = 2,
    )

    res.toString should be(
      """{"data":{"findManyB":[{"as":[{"c":{"id":2}},{"c":{"id":1}}]},{"as":[{"c":{"id":3}}]},{"as":[{"c":{"id":4}}]},{"as":[{"c":{"id":5}}]}]}}""".stripMargin
    )
  }
}
