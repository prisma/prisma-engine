package queries.aggregation

import org.scalatest.{FlatSpec, Matchers}
import play.api.libs.json.JsNull
import util._

class AverageAggregationQuerySpec extends FlatSpec with Matchers with ApiSpecBase {
  val project = SchemaDsl.fromStringV11() {
    """model Item {
      |  id    String @id @default(cuid())
      |  float Float
      |  int   Int
      |  dec   Decimal
      |  bInt  BigInt  
      |}
    """.stripMargin
  }

  override protected def beforeEach(): Unit = {
    super.beforeEach()
    database.setup(project)
  }

  def createItem(float: Double, int: Int, dec: String, bInt: String, id: Option[String] = None) = {
    val idString = id match {
      case Some(i) => s"""id: "$i","""
      case None    => ""
    }

    server.query(
      s"""mutation {
         |  createItem(data: { $idString float: $float, int: $int, dec: $dec, bInt: $bInt }) {
         |    id
         |  }
         |}""".stripMargin,
      project
    )
  }

  "Averaging with no records in the database" should "return null" in {
    val result = server.query(
      s"""{
         |  aggregateItem {
         |    avg {
         |      float
         |      int
         |      dec
         |      bInt
         |    }
         |  }
         |}""".stripMargin,
      project
    )

    result.pathAsJsValue("data.aggregateItem.avg.float") should be(JsNull)
    result.pathAsJsValue("data.aggregateItem.avg.dec") should be(JsNull)

    result.pathAsJsValue("data.aggregateItem.avg.int") should be(JsNull)
    result.pathAsJsValue("data.aggregateItem.avg.bInt") should be(JsNull)
  }

  "Averaging with some records in the database" should "return the correct averages" in {
    createItem(5.5, 5, "5.5", "5")
    createItem(4.5, 10, "4.5", "10")

    val result = server.query(
      s"""{
         |  aggregateItem {
         |    avg {
         |      float
         |      int
         |      dec
         |      bInt
         |    }
         |  }
         |}""".stripMargin,
      project
    )

    result.pathAsDouble("data.aggregateItem.avg.float") should be(5.0)
    result.pathAsString("data.aggregateItem.avg.dec") should be("5")

    result.pathAsDouble("data.aggregateItem.avg.int") should be(7.5)
    result.pathAsDouble("data.aggregateItem.avg.bInt") should be(7.5)
  }

  "Averaging with all sorts of query arguments" should "work" in {
    createItem(5.5, 5, "5.5", "5", Some("1"))
    createItem(4.5, 10, "4.5", "10", Some("2"))
    createItem(1.5, 2, "1.5", "2", Some("3"))
    createItem(0.0, 1, "0.0", "1", Some("4"))

    var result = server.query(
      """{
        |  aggregateItem(take: 2) {
        |    avg {
        |      float
        |      int
        |      dec
        |      bInt
        |    }
        |  }
        |}
      """.stripMargin,
      project
    )

    result.pathAsDouble("data.aggregateItem.avg.float") should be(5.0)
    result.pathAsString("data.aggregateItem.avg.dec") should be("5")
    result.pathAsDouble("data.aggregateItem.avg.int") should be(7.5)
    result.pathAsDouble("data.aggregateItem.avg.bInt") should be(7.5)

    result = server.query(
      """{
        |  aggregateItem(take: 5) {
        |    avg {
        |      float
        |      int
        |      dec
        |      bInt
        |    }
        |  }
        |}
      """.stripMargin,
      project
    )

    result.pathAsDouble("data.aggregateItem.avg.float") should be(2.875)
    result.pathAsString("data.aggregateItem.avg.dec") should be("2.875")
    result.pathAsDouble("data.aggregateItem.avg.int") should be(4.5)
    result.pathAsDouble("data.aggregateItem.avg.bInt") should be(4.5)

    result = server.query(
      """{
        |  aggregateItem(take: -5) {
        |    avg {
        |      float
        |      int
        |      dec
        |      bInt
        |    }
        |  }
        |}
      """.stripMargin,
      project
    )

    result.pathAsDouble("data.aggregateItem.avg.float") should be(2.875)
    result.pathAsString("data.aggregateItem.avg.dec") should be("2.875")
    result.pathAsDouble("data.aggregateItem.avg.int") should be(4.5)
    result.pathAsDouble("data.aggregateItem.avg.bInt") should be(4.5)

    result = server.query(
      """{
        |  aggregateItem(where: { id: { gt: "2" }}) {
        |    avg {
        |      float
        |      int
        |      dec
        |      bInt
        |    }
        |  }
        |}
      """.stripMargin,
      project
    )

    result.pathAsDouble("data.aggregateItem.avg.float") should be(0.75)
    result.pathAsString("data.aggregateItem.avg.dec") should be("0.75")
    result.pathAsDouble("data.aggregateItem.avg.int") should be(1.5)
    result.pathAsDouble("data.aggregateItem.avg.bInt") should be(1.5)

    result = server.query(
      """{
        |  aggregateItem(skip: 2) {
        |    avg {
        |      float
        |      int
        |      dec
        |      bInt
        |    }
        |  }
        |}
      """.stripMargin,
      project
    )

    result.pathAsDouble("data.aggregateItem.avg.float") should be(0.75)
    result.pathAsString("data.aggregateItem.avg.dec") should be("0.75")
    result.pathAsDouble("data.aggregateItem.avg.int") should be(1.5)
    result.pathAsDouble("data.aggregateItem.avg.bInt") should be(1.5)

    result = server.query(
      s"""{
        |  aggregateItem(cursor: { id: "3" }) {
        |    avg {
        |      float
        |      int
        |      dec
        |      bInt
        |    }
        |  }
        |}
      """.stripMargin,
      project
    )

    result.pathAsDouble("data.aggregateItem.avg.float") should be(0.75)
    result.pathAsString("data.aggregateItem.avg.dec") should be("0.75")
    result.pathAsDouble("data.aggregateItem.avg.int") should be(1.5)
    result.pathAsDouble("data.aggregateItem.avg.bInt") should be(1.5)
  }
}
