package writes.dataTypes.json

import org.scalatest.{FlatSpec, Matchers}
import util._

class JsonSpec extends FlatSpec with Matchers with ApiSpecBase {
  "Using a json field" should "work" taggedAs (IgnoreMySql56, IgnoreSQLite, IgnoreMsSql) in {
    val project = ProjectDsl.fromString {
      """|model Model {
         | id    String @id
         | field Json?  @default("{}")
         |}"""
    }

    database.setup(project)

    var res = server.query(
      s"""
         |mutation {
         |  createOneModel(
         |    data: {
         |      id: "A"
         |    }
         |  ) {
         |    field
         |  }
         |}""".stripMargin,
      project,
      legacy = false
    )

    res.toString should be("""{"data":{"createOneModel":{"field":"{}"}}}""")

    res = server.query(
      s"""
         |mutation {
         |  updateOneModel(
         |    where: { id: "A" }
         |    data: {
         |      field: "{\\"a\\":\\"b\\"}"
         |    }
         |  ) {
         |    field
         |  }
         |}""".stripMargin,
      project,
      legacy = false
    )

    res.toString should be("""{"data":{"updateOneModel":{"field":"{\"a\":\"b\"}"}}}""")

    res = server.query(
      s"""
         |mutation {
         |  updateOneModel(
         |    where: { id: "A" }
         |    data: {
         |      field: null
         |    }
         |  ) {
         |    field
         |  }
         |}""".stripMargin,
      project,
      legacy = false
    )

    res.toString should be("""{"data":{"updateOneModel":{"field":null}}}""")
  }
}
